import { Tabs } from 'antd'
import Task1CacheExperiment from './Task1-CacheExperiment'
import Task2AhooksDemo from './Task2-AhooksDemo'
import Task3BundleAnalysis from './Task3-BundleAnalysis'

export default function App() {
  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24 }}>
      <h1>Lesson 04 — 性能优化</h1>
      <Tabs
        items={[
          { key: 't1', label: '任务1: react-query 缓存实验', children: <Task1CacheExperiment /> },
          { key: 't2', label: '任务2: ahooks 集成', children: <Task2AhooksDemo /> },
          { key: 't3', label: '任务3: Bundle 分析 + React.lazy', children: <Task3BundleAnalysis /> },
        ]}
      />
    </div>
  )
}
