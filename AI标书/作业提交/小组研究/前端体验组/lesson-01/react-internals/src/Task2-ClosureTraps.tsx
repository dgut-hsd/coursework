import { useState, useEffect, useCallback, memo, useRef } from 'react'

// ==================== 陷阱 1：setInterval 读到旧值 ====================

function Trap1Broken() {
  const [count, setCount] = useState(0)
  useEffect(() => {
    // 空依赖 → count 被捕获为 0，每次 setCount(0 + 1) = 1
    const id = setInterval(() => setCount(count + 1), 1000)
    return () => clearInterval(id)
  }, [])
  return <p>Broken: count = {count}（永远停在 1）</p>
}

function Trap1Fixed() {
  const [count, setCount] = useState(0)
  useEffect(() => {
    // 修复：函数式更新，不依赖闭包中的 count
    const id = setInterval(() => setCount(c => c + 1), 1000)
    return () => clearInterval(id)
  }, [])
  return <p>Fixed: count = {count}</p>
}

// ==================== 陷阱 2：useEffect 空依赖导致状态过期 ====================

function Trap2Broken() {
  const [count, setCount] = useState(0)
  const [message, setMessage] = useState('等待 3 秒...')
  useEffect(() => {
    // 空依赖 → count 永远是 0
    const timer = setTimeout(() => {
      setMessage(`定时器触发时 count = ${count}`)
    }, 3000)
    return () => clearTimeout(timer)
  }, [])
  return (
    <div>
      <p>count: {count} <button onClick={() => setCount(c => c + 1)}>+1</button></p>
      <p>消息: {message}</p>
      <small>先点几次 +1，等 3 秒后看消息里的 count 值</small>
    </div>
  )
}

function Trap2Fixed() {
  const [count, setCount] = useState(0)
  const [message, setMessage] = useState('等待 3 秒...')
  useEffect(() => {
    // 修复：将 count 加入依赖数组
    const timer = setTimeout(() => {
      setMessage(`定时器触发时 count = ${count}`)
    }, 3000)
    return () => clearTimeout(timer)
  }, [count])
  return (
    <div>
      <p>count: {count} <button onClick={() => setCount(c => c + 1)}>+1</button></p>
      <p>消息: {message}</p>
    </div>
  )
}

// ==================== 陷阱 3：useCallback 引用不稳定 ====================

const MemoizedChild = memo(function Child({ onClick }: { onClick: () => void }) {
  const renders = useRef(0)
  renders.current++
  return (
    <div style={{ padding: 8, border: '1px dashed #999', marginTop: 4 }}>
      子组件渲染次数: {renders.current}
      <button onClick={onClick} style={{ marginLeft: 8 }}>click</button>
    </div>
  )
})

function Trap3Broken() {
  const [count, setCount] = useState(0)
  const [text, setText] = useState('')
  // useCallback 依赖 count → count 变化时引用变化 → 子组件重渲染
  const handleClick = useCallback(() => {
    console.log('count =', count)
  }, [count])
  return (
    <div>
      <input
        value={text}
        onChange={e => setText(e.target.value)}
        placeholder="输入文字"
      />
      <button onClick={() => setCount(c => c + 1)} style={{ marginLeft: 8 }}>count: {count}</button>
      <MemoizedChild onClick={handleClick} />
      <small>输入文字时子组件不重渲染 ✓<br />但点 +1 时子组件重渲染 ✗（因为 useCallback 依赖 count）</small>
    </div>
  )
}

function Trap3Fixed() {
  const [count, setCount] = useState(0)
  const [text, setText] = useState('')
  // 修复：用 ref 保存最新值，useCallback 空依赖 → 引用永远稳定
  const countRef = useRef(count)
  countRef.current = count
  const handleClick = useCallback(() => {
    console.log('count =', countRef.current)
  }, [])
  return (
    <div>
      <input
        value={text}
        onChange={e => setText(e.target.value)}
        placeholder="输入文字"
      />
      <button onClick={() => setCount(c => c + 1)} style={{ marginLeft: 8 }}>count: {count}</button>
      <MemoizedChild onClick={handleClick} />
      <small>点 +1 时子组件不再重渲染 ✓（callback 引用稳定）</small>
    </div>
  )
}

// ==================== 页面 ====================

function Section({ title, broken, fixed }: { title: string; broken: React.ReactNode; fixed: React.ReactNode }) {
  return (
    <div style={{ marginBottom: 16, padding: 12, border: '1px solid #ddd', borderRadius: 8 }}>
      <h3 style={{ marginTop: 0 }}>{title}</h3>
      <div style={{ display: 'flex', gap: 24 }}>
        <div style={{ flex: 1 }}>
          <strong style={{ color: '#f5222d' }}>❌ 陷阱</strong>
          {broken}
        </div>
        <div style={{ flex: 1 }}>
          <strong style={{ color: '#52c41a' }}>✅ 修复</strong>
          {fixed}
        </div>
      </div>
    </div>
  )
}

export default function Task2ClosureTraps() {
  return (
    <div>
      <Section title="陷阱 1：setInterval 读到旧值" broken={<Trap1Broken />} fixed={<Trap1Fixed />} />
      <Section title="陷阱 2：useEffect 空依赖导致状态过期" broken={<Trap2Broken />} fixed={<Trap2Fixed />} />
      <Section title="陷阱 3：useCallback 引用不稳定" broken={<Trap3Broken />} fixed={<Trap3Fixed />} />
    </div>
  )
}
