import { useState } from 'react'
import { Button, Progress, List, Tag, Card, Empty } from 'antd'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeRaw from 'rehype-raw'
import { useAuditSSE } from './useAuditSSE'

export default function Task3SSEMarkdown() {
  const [taskId, setTaskId] = useState<string | null>(null)
  const { messages, state } = useAuditSSE(taskId)

  const progress = messages.filter(m => m.type === 'progress')
  const findings = messages.filter(m => m.type === 'finding')
  const complete = messages.find(m => m.type === 'complete')
  const lastProgress = progress[progress.length - 1]

  return (
    <div>
      <Button
        type="primary"
        onClick={() => setTaskId('task-' + Date.now())}
        disabled={state === 'connecting' || state === 'open'}
      >
        {state === 'idle' ? '开始审核' : state === 'connecting' ? '连接中...' : state === 'open' ? '审核中...' : '审核完成，可重新开始'}
      </Button>

      {lastProgress && (
        <div style={{ marginTop: 16 }}>
          <p>当前阶段: {lastProgress.data.currentStage}</p>
          <Progress percent={lastProgress.data.progress} />
        </div>
      )}

      {findings.length > 0 && (
        <Card title="发现的问题" size="small" style={{ marginTop: 16 }}>
          <List
            dataSource={findings}
            renderItem={(f: any) => (
              <List.Item>
                <Tag color={f.data.severity === 'critical' ? 'error' : 'warning'}>
                  {f.data.severity}
                </Tag>
                {f.data.title} — {f.data.description}
              </List.Item>
            )}
          />
        </Card>
      )}

      {complete ? (
        <Card title="审核报告 (react-markdown)" size="small" style={{ marginTop: 16 }}>
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeRaw]}
            components={{
              table: ({ children }) => <table style={{ borderCollapse: 'collapse', width: '100%' }}>{children}</table>,
              th: ({ children }) => <th style={{ border: '1px solid #ddd', padding: 8, background: '#fafafa' }}>{children}</th>,
              td: ({ children }) => <td style={{ border: '1px solid #ddd', padding: 8 }}>{children}</td>,
              code: ({ children }) => {
                const text = String(children)
                if (text.startsWith('《') && text.endsWith('》')) {
                  return <code style={{ color: '#cf1322', fontWeight: 600 }}>{children}</code>
                }
                return <code>{children}</code>
              },
            }}
          >
            {complete.data.report}
          </ReactMarkdown>
        </Card>
      ) : state !== 'idle' && !complete && (
        <Empty description="等待审核完成..." style={{ marginTop: 24 }} />
      )}
    </div>
  )
}
