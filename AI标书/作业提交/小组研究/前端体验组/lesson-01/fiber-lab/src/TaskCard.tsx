import { useState, useEffect, useCallback } from 'react'

interface TaskCardProps {
  id: number
  title: string
}

function TaskCard({ id, title }: TaskCardProps) {
  const [status, setStatus] = useState<'PENDING' | 'APPROVED' | 'REJECTED'>('PENDING')
  const [comment, setComment] = useState('')

  // 这个 effect 是为了让 hooks 链表更长，方便观察
  useEffect(() => {
    console.log(`Task ${id} mounted`)
    return () => {
      console.log(`Task ${id} unmounted`)
    }
  }, [id])

  const handleApprove = useCallback(() => {
    setStatus('APPROVED')
  }, [])

  const handleReject = useCallback(() => {
    setStatus('REJECTED')
  }, [])

  return (
    <div className="task-card" style={{ border: '1px solid #ccc', padding: 12, margin: 8 }}>
      <h3>{title} (#{id})</h3>
      <p>
        状态：<strong style={{ color: status === 'APPROVED' ? 'green' : status === 'REJECTED' ? 'red' : 'orange' }}>
          {status}
        </strong>
      </p>
      <input
        type="text"
        placeholder="输入审核意见..."
        value={comment}
        onChange={(e) => setComment(e.target.value)}
        style={{ display: 'block', margin: '8px 0', padding: 4 }}
      />
      <div>
        <button onClick={handleApprove} disabled={status === 'APPROVED'}>
          ✅ 通过
        </button>
        <button onClick={handleReject} disabled={status === 'REJECTED'} style={{ marginLeft: 8 }}>
          ❌ 驳回
        </button>
      </div>
    </div>
  )
}

export default TaskCard
