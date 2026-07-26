use lru::LruCache;
use serde_json::Value;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};

pub type SearchResult = Value;

pub trait Searcher: Send + Sync + Clone {
    fn search(&self, query: &str) -> tokio::task::JoinHandle<SearchResult>;
}

#[derive(Clone)]
pub struct MockSearcher {
    call_count: Arc<std::sync::atomic::AtomicUsize>,
    delay_ms: u64,
}

impl MockSearcher {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            call_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            delay_ms,
        }
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Searcher for MockSearcher {
    fn search(&self, query: &str) -> tokio::task::JoinHandle<SearchResult> {
        let call_count = Arc::clone(&self.call_count);
        let delay_ms = self.delay_ms;
        let query_clone = query.to_string();
        
        tokio::spawn(async move {
            call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            
            serde_json::json!({
                "query": query_clone,
                "results": [
                    {"id": "1", "content": format!("Result for: {}", query_clone)},
                    {"id": "2", "content": "More results"}
                ]
            })
        })
    }
}

pub struct SearchBuffer {
    pending: Arc<RwLock<HashMap<String, Arc<Mutex<Option<SearchResult>>>>>>,
    cache: Arc<RwLock<LruCache<String, SearchResult>>>,
}

impl SearchBuffer {
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(cache_capacity).unwrap()))),
        }
    }

    pub async fn search<T: Searcher + ?Sized>(&self, query: &str, searcher: &T) -> SearchResult {
        let key = normalize_query(query);

        {
            let mut cache = self.cache.write().await;
            if let Some(cached) = cache.get(&key) {
                return cached.clone();
            }
        }

        {
            let pending = self.pending.read().await;
            if let Some(mutex) = pending.get(&key) {
                let mutex_clone = Arc::clone(mutex);
                drop(pending);
                let mut lock = mutex_clone.lock().await;
                while lock.is_none() {
                    drop(lock);
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    lock = mutex_clone.lock().await;
                }
                return lock.as_ref().unwrap().clone();
            }
        }

        let mutex = Arc::new(Mutex::new(None));
        self.pending.write().await.insert(key.clone(), Arc::clone(&mutex));

        let query_clone = query.to_string();
        let _searcher_clone = searcher.clone();

        let inner_handle = searcher.search(&query_clone);
        let result = inner_handle.await.expect("search task failed");

        {
            let mut lock = mutex.lock().await;
            *lock = Some(result.clone());
        }

        self.cache.write().await.put(key.clone(), result.clone());
        self.pending.write().await.remove(&key);

        result
    }

    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }

    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }
}

fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_buffer_single_request() {
        let buffer = SearchBuffer::new(10);
        let searcher = MockSearcher::new(10);

        let result = buffer.search("test query", &searcher).await;
        assert!(result.get("query").is_some());
    }

    #[tokio::test]
    async fn test_search_buffer_concurrent_deduplication() {
        let buffer = Arc::new(SearchBuffer::new(10));
        let searcher = MockSearcher::new(50);

        let futures = vec![
            tokio::spawn({
                let buffer = Arc::clone(&buffer);
                let searcher = searcher.clone();
                async move { buffer.search("same query", &searcher).await }
            }),
            tokio::spawn({
                let buffer = Arc::clone(&buffer);
                let searcher = searcher.clone();
                async move { buffer.search("same query", &searcher).await }
            }),
            tokio::spawn({
                let buffer = Arc::clone(&buffer);
                let searcher = searcher.clone();
                async move { buffer.search("same query", &searcher).await }
            }),
        ];

        let results = futures::future::join_all(futures).await;
        assert!(results.iter().all(|r| r.is_ok()));

        // 核心验收：3 个并发请求只触发 1 次 HTTP 调用
        assert_eq!(searcher.call_count(), 1, "3 个并发相同查询应只发 1 次 HTTP 请求");

        // 验证 3 个结果内容相同
        let vals: Vec<Value> = results.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(vals[0], vals[1]);
        assert_eq!(vals[1], vals[2]);
    }

    #[tokio::test]
    async fn test_search_buffer_caching() {
        let buffer = SearchBuffer::new(10);
        let searcher = MockSearcher::new(10);

        let _ = buffer.search("cache test", &searcher).await;
        let _ = buffer.search("cache test", &searcher).await;
        let _ = buffer.search("cache test", &searcher).await;

        assert_eq!(buffer.cache_size().await, 1);
    }

    #[tokio::test]
    async fn test_search_buffer_different_queries() {
        let buffer = SearchBuffer::new(10);
        let searcher = MockSearcher::new(10);

        let _ = buffer.search("query 1", &searcher).await;
        let _ = buffer.search("query 2", &searcher).await;
        let _ = buffer.search("query 3", &searcher).await;

        assert_eq!(buffer.cache_size().await, 3);
    }

    #[tokio::test]
    async fn test_search_buffer_normalization() {
        let buffer = SearchBuffer::new(10);
        let searcher = MockSearcher::new(10);

        let _ = buffer.search("  Test Query  ", &searcher).await;
        let _ = buffer.search("test query", &searcher).await;
        let _ = buffer.search("TEST QUERY", &searcher).await;

        assert_eq!(buffer.cache_size().await, 1);
    }
}
