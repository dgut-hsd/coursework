import { useRef } from 'react'
import { Card, Input, List, Spin, Button } from 'antd'
import { useRequest, useDebounceFn, useInfiniteScroll } from 'ahooks'

// ① useRequest：替代手动 useState + useEffect
function useSearchLaws() {
  return useRequest(
    (keyword: string) =>
      new Promise<{ id: string; name: string }[]>(resolve => {
        setTimeout(() => {
          if (!keyword) return resolve([])
          const mock = ['招标投标法', '安全生产法', '建筑法', '政府采购法']
            .filter(l => l.includes(keyword))
            .map((name, i) => ({ id: String(i), name }))
          resolve(mock)
        }, 400)
      }),
    {
      debounceWait: 300, // 输入停止 300ms 后才请求
    },
  )
}

// ③ useInfiniteScroll：无限滚动
function useAuditHistory() {
  const result = useInfiniteScroll(
    (d) =>
      new Promise<{ list: { id: string; title: string }[]; hasMore: boolean }>(resolve => {
        setTimeout(() => {
          const page = d?.list?.length ?? 0
          const pageSize = 5
          const list = Array.from({ length: pageSize }, (_, i) => ({
            id: String(page + i),
            title: `审核记录 ${page + i + 1}`,
          }))
          resolve({ list, hasMore: page + pageSize < 20 })
        }, 500)
      }),
  )
  return result
}

function SearchDemo() {
  const { data, loading, run } = useSearchLaws()
  return (
    <Card title="① useRequest + debounceWait" size="small">
      <Input.Search
        placeholder="输入法规名称（如：招标）"
        onChange={e => run(e.target.value)}
        enterButton
      />
      {loading && <Spin size="small" style={{ margin: 8 }} />}
      <List
        size="small"
        dataSource={data || []}
        renderItem={(item) => <List.Item>{item.name}</List.Item>}
      />
    </Card>
  )
}

function DebounceDemo() {
  const countRef = useRef(0)
  const { run, cancel } = useDebounceFn(
    () => {
      countRef.current++
      console.log('debounced call:', countRef.current)
    },
    { wait: 500 },
  )
  return (
    <Card title="② useDebounceFn（500ms 防抖）" size="small">
      <p>连续点击只会在停止后执行一次，查看 console</p>
      <Button onClick={run} style={{ marginRight: 8 }}>快速点击</Button>
      <Button onClick={cancel}>取消</Button>
    </Card>
  )
}

function InfiniteScrollDemo() {
  const { data, loading, loadMore, loadingMore, noMore } = useAuditHistory()
  return (
    <Card title="③ useInfiniteScroll（审核历史）" size="small">
      <List
        size="small"
        dataSource={data?.list || []}
        renderItem={(item) => <List.Item>{item.title}</List.Item>}
      />
      {loading && <Spin style={{ display: 'block', margin: '8px auto' }} />}
      {!loading && !noMore && (
        <Button onClick={loadMore} loading={loadingMore} style={{ marginTop: 8 }}>
          加载更多
        </Button>
      )}
      {noMore && <p style={{ color: '#999', textAlign: 'center' }}>没有更多了</p>}
    </Card>
  )
}

export default function Task2AhooksDemo() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <SearchDemo />
      <DebounceDemo />
      <InfiniteScrollDemo />
    </div>
  )
}
