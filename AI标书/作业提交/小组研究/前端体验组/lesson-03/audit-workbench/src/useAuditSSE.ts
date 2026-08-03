import { useState, useEffect } from 'react'
import { startMockSSE, type SSEMessage } from './mockSSE'

// 实际项目中：
// import { EventSource } from 'eventsource'
// const es = new EventSource(`/api/audit/tasks/${taskId}/stream`)
// es.addEventListener('progress', ...)
// es.addEventListener('finding', ...)
// es.addEventListener('complete', ...)
// es.onerror = () => { es.close() }
// return () => es.close()

export function useAuditSSE(taskId: string | null) {
  const [messages, setMessages] = useState<SSEMessage[]>([])
  const [state, setState] = useState<'idle' | 'connecting' | 'open' | 'closed'>('idle')

  useEffect(() => {
    if (!taskId) return

    setState('connecting')
    setMessages([])

    // Mock：用 setTimeout 模拟 SSE。实际项目替换为 new EventSource(url)
    const cleanup = startMockSSE((msg) => {
      setState('open')
      setMessages(prev => [...prev, msg])
      if (msg.type === 'complete') setState('closed')
    })

    return cleanup
  }, [taskId])

  return { messages, state }
}
