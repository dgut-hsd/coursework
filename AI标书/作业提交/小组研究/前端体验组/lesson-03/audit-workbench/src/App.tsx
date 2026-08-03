import { Tabs } from 'antd'
import Task1PdfHighlight from './Task1-PdfHighlight'
import Task2EchartsDashboard from './Task2-EchartsDashboard'
import Task3SSEMarkdown from './Task3-SSEMarkdown'

export default function App() {
  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24 }}>
      <h1>Lesson 03 — 审核工作台</h1>
      <Tabs
        items={[
          { key: 't1', label: '任务1: react-pdf + bbox 高亮', children: <Task1PdfHighlight /> },
          { key: 't2', label: '任务2: echarts Dashboard', children: <Task2EchartsDashboard /> },
          { key: 't3', label: '任务3: SSE + react-markdown', children: <Task3SSEMarkdown /> },
        ]}
      />
    </div>
  )
}
