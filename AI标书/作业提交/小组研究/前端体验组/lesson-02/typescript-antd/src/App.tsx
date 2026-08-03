import { Tabs } from 'antd'
import Task1AuthSlice from './Task1-AuthSlice'
import Task2QueryCache from './Task2-QueryCache'
import Task3IssueCard from './Task3-IssueCard'

export default function App() {
  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24 }}>
      <h1>Lesson 02 — TypeScript + Ant Design</h1>
      <Tabs
        items={[
          { key: 't1', label: '任务1: Redux auth slice', children: <Task1AuthSlice /> },
          { key: 't2', label: '任务2: react-query 缓存联动', children: <Task2QueryCache /> },
          { key: 't3', label: '任务3: antd-style IssueCard', children: <Task3IssueCard /> },
        ]}
      />
    </div>
  )
}
