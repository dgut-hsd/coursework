import { useState } from 'react'
import { Card, Button, Input, List, Tag, Space, Segmented, message } from 'antd'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchProjects, fetchTasks, createTask } from './api/mockApi'

function Dashboard() {
  const { data: projects, isLoading } = useQuery({
    queryKey: ['projects'],
    queryFn: fetchProjects,
    staleTime: 5 * 60 * 1000,
  })

  return (
    <Card title="Dashboard（staleTime=5min）">
      <p>切换到审核工作台创建任务后切回来，列表会因 invalidateQueries 自动刷新</p>
      <List
        loading={isLoading}
        dataSource={projects || []}
        renderItem={p => (
          <List.Item>
            <Space>
              <Tag color={p.status === 'active' ? 'green' : 'default'}>{p.status}</Tag>
              {p.name}
            </Space>
          </List.Item>
        )}
      />
    </Card>
  )
}

function Workbench() {
  const queryClient = useQueryClient()
  const [selectedProject, setSelectedProject] = useState('p1')
  const [newTaskTitle, setNewTaskTitle] = useState('')

  const { data: projects } = useQuery({
    queryKey: ['projects'],
    queryFn: fetchProjects,
    staleTime: 5 * 60 * 1000,
  })

  const { data: tasks, isLoading } = useQuery({
    queryKey: ['tasks', selectedProject],
    queryFn: () => fetchTasks(selectedProject),
    staleTime: 10 * 1000,
  })

  const createMutation = useMutation({
    mutationFn: () => createTask(selectedProject, newTaskTitle),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks', selectedProject] })
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      message.success('任务创建成功')
      setNewTaskTitle('')
    },
  })

  return (
    <Card title="审核工作台">
      <Space style={{ marginBottom: 16 }}>
        <Segmented
          value={selectedProject}
          onChange={v => setSelectedProject(v as string)}
          options={(projects || []).map(p => ({ label: p.name, value: p.id }))}
        />
      </Space>

      <List
        loading={isLoading}
        dataSource={tasks || []}
        renderItem={t => (
          <List.Item>
            <Space>
              <Tag color={t.status === 'COMPLETED' ? 'green' : t.status === 'PROCESSING' ? 'blue' : 'orange'}>
                {t.status}
              </Tag>
              {t.title}
            </Space>
          </List.Item>
        )}
      />

      <Space.Compact style={{ marginTop: 16 }}>
        <Input
          value={newTaskTitle}
          onChange={e => setNewTaskTitle(e.target.value)}
          placeholder="新任务标题"
        />
        <Button
          type="primary"
          loading={createMutation.isPending}
          onClick={() => newTaskTitle && createMutation.mutate()}
        >
          创建任务
        </Button>
      </Space.Compact>
    </Card>
  )
}

export default function Task2QueryCache() {
  const [page, setPage] = useState<'dashboard' | 'workbench'>('dashboard')

  return (
    <div>
      <Segmented
        value={page}
        onChange={v => setPage(v as 'dashboard' | 'workbench')}
        options={[
          { label: 'Dashboard', value: 'dashboard' },
          { label: '审核工作台', value: 'workbench' },
        ]}
        style={{ marginBottom: 16 }}
      />
      {page === 'dashboard' ? <Dashboard /> : <Workbench />}
    </div>
  )
}
