use crate::tool_registry::{ToolRegistry, SearchKnowledgeTool, ReadSectionTool, OutputFindingTool};
use crate::search_buffer::{SearchBuffer, MockSearcher, Searcher};
use crate::error_isolation::{execute_tool_safe, execute_tool_with_retry, ToolResult};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

pub async fn run_demo() {
    println!("=== Lesson-02 工具使用演示 ===");
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 1：工具注册表演示");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_tool_registry().await;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 2：搜索缓冲器演示（并发去重）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_search_buffer().await;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 3：错误隔离演示（超时/重试）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_error_isolation().await;
}

async fn demo_tool_registry() {
    let mut registry = ToolRegistry::new();
    
    registry.register(Box::new(SearchKnowledgeTool));
    registry.register(Box::new(ReadSectionTool));
    registry.register(Box::new(OutputFindingTool));
    
    println!("已注册工具: {:?}", registry.tool_names());
    
    let defs = registry.definitions();
    for def in defs {
        println!("\n工具: {}", def.name);
        println!("  描述: {}", def.description);
        println!("  参数: {:?}", def.parameters);
    }
    
    println!("\n→ 测试搜索法律知识工具");
    let search_result = registry.execute("search_knowledge", json!({"query": "建筑资质"})).await.unwrap();
    println!("  查询: {}", search_result["query"]);
    println!("  结果数: {}", search_result["total"]);
    for (i, result) in search_result["results"].as_array().unwrap().iter().enumerate() {
        println!("    结果{}: {} - {}", i + 1, result["title"], result["article"]);
    }
    
    println!("\n→ 测试读取条款工具");
    let read_result = registry.execute("read_section", json!({"clause_id": "cl_042"})).await.unwrap();
    println!("  条款ID: {}", read_result["clause_id"]);
    println!("  内容: {}", read_result["content"]);
    
    println!("\n→ 测试输出结果工具");
    let output_result = registry.execute("output_finding", json!({
        "summary": "测试完成",
        "compliant": true
    })).await.unwrap();
    println!("  摘要: {}", output_result["summary"]);
    println!("  合规: {}", output_result["compliant"]);
    println!("  时间: {}", output_result["timestamp"]);
    
    println!("\n✓ 工具注册表演示完成");
}

async fn demo_search_buffer() {
    let buffer = Arc::new(SearchBuffer::new(10));
    let searcher = MockSearcher::new(50);
    
    println!("模拟：3个并发请求查询相同内容");
    println!("搜索器初始调用次数: {}", searcher.call_count());
    
    let futures = vec![
        tokio::spawn({
            let buffer = Arc::clone(&buffer);
            let searcher = searcher.clone();
            async move { buffer.search("同样的查询", &searcher).await }
        }),
        tokio::spawn({
            let buffer = Arc::clone(&buffer);
            let searcher = searcher.clone();
            async move { buffer.search("同样的查询", &searcher).await }
        }),
        tokio::spawn({
            let buffer = Arc::clone(&buffer);
            let searcher = searcher.clone();
            async move { buffer.search("同样的查询", &searcher).await }
        }),
    ];
    
    let results = futures::future::join_all(futures).await;
    let vals: Vec<serde_json::Value> = results.into_iter().map(|r| r.unwrap()).collect();
    
    println!("搜索器最终调用次数: {}", searcher.call_count());
    println!("缓存大小: {}", buffer.cache_size().await);
    println!("pending 请求数: {}", buffer.pending_count().await);
    
    println!("\n验证3个结果是否相同:");
    println!("  结果1: {}", vals[0]["query"]);
    println!("  结果2: {}", vals[1]["query"]);
    println!("  结果3: {}", vals[2]["query"]);
    println!("  全部相同: {}", vals[0] == vals[1] && vals[1] == vals[2]);
    
    println!("\n✓ 搜索缓冲器演示完成（3个并发请求只触发1次HTTP调用）");
}

async fn demo_error_isolation() {
    let search_tool = crate::tool_registry::SearchKnowledgeTool;
    
    println!("→ 测试正常执行");
    let result = execute_tool_safe(&search_tool, &json!({"query": "test"}), Duration::from_secs(5)).await;
    match result {
        ToolResult::Success(v) => println!("  ✓ 成功: {}", v["query"]),
        _ => println!("  ✗ 失败"),
    }
    
    println!("\n→ 测试超时（工具设置100%超时概率）");
    let flaky_tool = crate::error_isolation::FlakySearchTool::new(0.0, 1.0);
    let result = execute_tool_safe(&flaky_tool, &json!({"query": "test"}), Duration::from_millis(100)).await;
    match result {
        ToolResult::Timeout(name) => println!("  ✓ 超时: {}", name),
        _ => println!("  ✗ 未超时"),
    }
    
    println!("\n→ 测试错误（工具设置100%失败概率）");
    let error_tool = crate::error_isolation::FlakySearchTool::new(1.0, 0.0);
    let result = execute_tool_safe(&error_tool, &json!({"query": "test"}), Duration::from_secs(5)).await;
    match result {
        ToolResult::Error(msg) => println!("  ✓ 错误捕获: {}", msg),
        _ => println!("  ✗ 未捕获错误"),
    }
    
    println!("\n→ 测试重试机制（50%失败概率，最多重试3次）");
    let retry_tool = crate::error_isolation::FlakySearchTool::new(0.5, 0.0);
    let result = execute_tool_with_retry(&retry_tool, &json!({"query": "test"}), Duration::from_secs(1), 3).await;
    match result {
        ToolResult::Success(_) => println!("  ✓ 重试后成功"),
        ToolResult::Error(msg) => println!("  ✗ 重试后仍失败: {}", msg),
        _ => println!("  ✗ 其他结果"),
    }
    
    println!("\n✓ 错误隔离演示完成");
}
