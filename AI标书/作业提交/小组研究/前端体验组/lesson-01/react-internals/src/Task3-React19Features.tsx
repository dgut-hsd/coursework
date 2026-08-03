import { useState, use, useOptimistic, useTransition, useEffect, Suspense } from 'react'

// ==================== 类型 & Mock ====================

interface AuditTask {
  id: string
  title: string
  status: 'PENDING' | 'PROCESSING' | 'COMPLETED' | 'FAILED'
}

// 缓存 Promise（use() 要求同一个 Promise 引用）
const taskCache = new Map<string, Promise<AuditTask>>()
function fetchAuditTask(taskId: string): Promise<AuditTask> {
  if (!taskCache.has(taskId)) {
    taskCache.set(taskId, new Promise(resolve => {
      setTimeout(() => resolve({ id: taskId, title: '标书审核任务-001', status: 'COMPLETED' }), 1500)
    }))
  }
  return taskCache.get(taskId)!
}

function createAuditTask(): Promise<AuditTask> {
  return new Promise(resolve => {
    setTimeout(() => resolve({ id: 'real-' + Date.now(), title: '新建审核任务', status: 'COMPLETED' }), 1000)
  })
}

// ==================== React 18 写法 ====================

function AuditTaskV18() {
  const [task, setTask] = useState<AuditTask | null>(null)
  const [loading, setLoading] = useState(true)
  const [tasks, setTasks] = useState<AuditTask[]>([])
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    fetchAuditTask('task-001')
      .then(data => {
        if (!cancelled) { setTask(data); setLoading(false) }
      })
    return () => { cancelled = true }
  }, [])

  // 手动乐观更新：先加临时任务，成功替换，失败回滚
  const handleSubmit = async () => {
    setSubmitting(true)
    const tempId = 'temp-' + Date.now()
    setTasks(prev => [...prev, { id: tempId, title: '新建审核任务', status: 'PENDING' }])
    try {
      const realTask = await createAuditTask()
      setTasks(prev => prev.map(t => t.id === tempId ? realTask : t))
    } catch {
      setTasks(prev => prev.filter(t => t.id !== tempId))
    } finally {
      setSubmitting(false)
    }
  }

  if (loading) return <p>⏳ Loading...</p>
  if (!task) return null

  return (
    <div>
      <p>当前任务: {task.title}（{task.status}）</p>
      <button onClick={handleSubmit} disabled={submitting}>
        {submitting ? '提交中...' : '提交审核任务'}
      </button>
      <ul>{tasks.map(t => <li key={t.id}>{t.title} - {t.status}</li>)}</ul>
    </div>
  )
}

// ==================== React 19 写法 ====================

function AuditTaskV19Inner() {
  // use() 直接在渲染中读取 Promise，Suspense 自动处理 loading
  const task = use(fetchAuditTask('task-001'))

  const [tasks, setTasks] = useState<AuditTask[]>([])
  // useOptimistic 自动管理乐观状态 + 回滚
  const [optimisticTasks, addOptimistic] = useOptimistic(
    tasks,
    (state, newTask: AuditTask) => [...state, { ...newTask, status: 'PENDING' as const }],
  )
  const [isPending, startTransition] = useTransition()

  const handleSubmit = () => {
    const tempId = 'temp-' + Date.now()
    startTransition(async () => {
      addOptimistic({ id: tempId, title: '新建审核任务', status: 'PENDING' })
      const realTask = await createAuditTask()
      setTasks(prev => [...prev, realTask])
    })
  }

  return (
    <div>
      <p>当前任务: {task.title}（{task.status}）</p>
      <button onClick={handleSubmit} disabled={isPending}>
        {isPending ? '提交中...' : '提交审核任务'}
      </button>
      <ul>{optimisticTasks.map(t => <li key={t.id}>{t.title} - {t.status}</li>)}</ul>
    </div>
  )
}

function AuditTaskV19() {
  return (
    <Suspense fallback={<p>⏳ Loading (Suspense)...</p>}>
      <AuditTaskV19Inner />
    </Suspense>
  )
}

// ==================== 页面 ====================

export default function Task3React19Features() {
  return (
    <div>
      <p>左右对比：React 18 三段式 vs React 19 use() + useOptimistic</p>
      <div style={{ display: 'flex', gap: 24 }}>
        <div style={{ flex: 1, padding: 12, border: '1px solid #ddd', borderRadius: 8 }}>
          <h3 style={{ marginTop: 0 }}>React 18 写法</h3>
          <p style={{ fontSize: 13, color: '#666' }}>useState + useEffect + 手动 loading/error + 手动乐观更新</p>
          <AuditTaskV18 />
        </div>
        <div style={{ flex: 1, padding: 12, border: '1px solid #ddd', borderRadius: 8 }}>
          <h3 style={{ marginTop: 0 }}>React 19 写法</h3>
          <p style={{ fontSize: 13, color: '#666' }}>use() 读取 Promise + useOptimistic 自动乐观/回滚</p>
          <AuditTaskV19 />
        </div>
      </div>
    </div>
  )
}
