import { useState } from 'react'
import TaskCard from './TaskCard'
import './App.css'

function App() {
  const [count, setCount] = useState(0)

  return (
    <div>
      <h1>Fiber 树遍历实验</h1>
      <p>打开控制台粘贴遍历代码 👇</p>
      <button onClick={() => setCount((count) => count + 1)}>
        Count is {count}
      </button>

      <TaskCard id={1} title="标书审核-项目A" />
    </div>
  )
}

export default App
