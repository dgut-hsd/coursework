import Task1FiberTraversal from './Task1-FiberTraversal'
import Task2ClosureTraps from './Task2-ClosureTraps'
import Task3React19Features from './Task3-React19Features'

export default function App() {
  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24, fontFamily: 'system-ui, sans-serif' }}>
      <h1>Lesson 01 — React 19 内核</h1>
      <hr />
      <h2>任务 1：Fiber 树遍历</h2>
      <Task1FiberTraversal />
      <hr />
      <h2>任务 2：闭包陷阱实验室</h2>
      <Task2ClosureTraps />
      <hr />
      <h2>任务 3：React 19 新特性对比</h2>
      <Task3React19Features />
    </div>
  )
}
