import { useState } from 'react'
import { Card, Slider, Statistic, Row, Col, Button, List, Tag } from 'antd'
import { useQuery, useQueryClient } from '@tanstack/react-query'

// Mock API
let requestCount = 0
async function fetchProjects() {
  requestCount++
  return new Promise<{ id: string; name: string }[]>(resolve => {
    setTimeout(() => resolve([
      { id: 'p1', name: '市政工程标书' },
      { id: 'p2', name: '道路建设标书' },
      { id: 'p3', name: '桥梁施工标书' },
    ]), 500)
  })
}

async function fetchTasks() {
  requestCount++
  return new Promise<{ id: string; title: string; status: string }[]>(resolve => {
    setTimeout(() => resolve([
      { id: 't1', title: '资质审核', status: 'COMPLETED' },
      { id: 't2', title: '报价审核', status: 'PENDING' },
    ]), 500)
  })
}

function useProjects(staleTime: number) {
  return useQuery({
    queryKey: ['projects'],
    queryFn: fetchProjects,
    staleTime,
  })
}

function useTasks(staleTime: number) {
  return useQuery({
    queryKey: ['tasks'],
    queryFn: fetchTasks,
    staleTime,
  })
}

export default function Task1CacheExperiment() {
  const [staleTime, setStaleTime] = useState(5000)
  const [page, setPage] = useState<'projects' | 'tasks'>('projects')
  const queryClient = useQueryClient()

  const projectsQuery = useProjects(staleTime)
  const tasksQuery = useTasks(staleTime)
  const currentQuery = page === 'projects' ? projectsQuery : tasksQuery

  const handlePrefetch = () => {
    if (page === 'projects') {
      queryClient.prefetchQuery({
        queryKey: ['tasks'],
        queryFn: fetchTasks,
        staleTime,
      })
    } else {
      queryClient.prefetchQuery({
        queryKey: ['projects'],
        queryFn: fetchProjects,
        staleTime,
      })
    }
  }

  const data = page === 'projects' ? projectsQuery.data : tasksQuery.data

  return (
    <div>
      <Card title="staleTime 调节" size="small" style={{ marginBottom: 16 }}>
        <Row gutter={16} align="middle">
          <Col>
            <Statistic title="staleTime" value={staleTime / 1000} suffix="秒" />
          </Col>
          <Col flex="auto">
            <Slider
              min={0}
              max={30}
              step={1}
              value={staleTime / 1000}
              onChange={v => setStaleTime(v * 1000)}
            />
          </Col>
          <Col>
            <Statistic title="网络请求总数" value={requestCount} valueStyle={{ color: '#1677ff' }} />
          </Col>
        </Row>
      </Card>

      <Card
        title={
          <div>
            <Button
              type={page === 'projects' ? 'primary' : 'default'}
              size="small"
              onClick={() => setPage('projects')}
              style={{ marginRight: 8 }}
            >项目列表</Button>
            <Button
              type={page === 'tasks' ? 'primary' : 'default'}
              size="small"
              onClick={() => setPage('tasks')}
            >审核任务</Button>
            <Button size="small" onClick={handlePrefetch} style={{ marginLeft: 8 }}>预取另一页数据</Button>
          </div>
        }
      >
        <p>切换页面再切回来：staleTime 内不重新请求（缓存命中）</p>
        <List
          loading={currentQuery.isLoading}
          dataSource={data || []}
          renderItem={(item: any) => (
            <List.Item>
              <Tag>{item.id}</Tag> {item.name || item.title}
              {item.status && <Tag color="blue">{item.status}</Tag>}
            </List.Item>
          )}
        />
      </Card>
    </div>
  )
}
