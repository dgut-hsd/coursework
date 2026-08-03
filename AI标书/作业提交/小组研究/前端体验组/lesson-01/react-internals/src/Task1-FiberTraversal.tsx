import { useState, useEffect } from 'react'
import * as ReactNS from 'react'

// React Fiber tag 枚举（部分）
const FiberTagMap: Record<number, string> = {
  0: 'FunctionComponent',
  1: 'ClassComponent',
  3: 'HostRoot',
  5: 'HostComponent',
  6: 'HostText',
  7: 'Fragment',
  9: 'ContextConsumer',
  10: 'ContextProvider',
  11: 'ForwardRef',
  15: 'SimpleMemoComponent',
  18: 'SuspenseComponent',
  21: 'OffscreenComponent',
  26: 'ActivityComponent',
}

// 通过 DOM 元素的 __reactContainer$ 属性拿到 Fiber 根节点
function getFiberRoot(container: HTMLElement): any | null {
  const key = Object.keys(container).find(k => k.startsWith('__reactContainer$'))
  if (!key) return null
  const fiberRoot = (container as any)[key]
  return fiberRoot?.current ?? null
}

// 格式化 Hooks 链表
function formatHooks(memoizedState: any): string {
  if (!memoizedState) return ''
  if (typeof memoizedState === 'object' && 'next' in memoizedState) {
    const hooks: string[] = []
    let hook = memoizedState
    while (hook) {
      const state = hook.memoizedState
      let str: string
      if (state === null) str = 'null'
      else if (typeof state === 'function') str = 'fn'
      else if (typeof state === 'object') {
        try { str = JSON.stringify(state)?.slice(0, 50) ?? '[obj]' } catch { str = '[obj]' }
      } else str = String(state).slice(0, 50)
      hooks.push(str)
      hook = hook.next
    }
    return `  hooks: [${hooks.join(' → ')}]`
  }
  return ''
}

// 递归遍历 Fiber 树
function traverseFiber(fiber: any, depth: number, lines: string[]) {
  if (!fiber) return

  const indent = '  '.repeat(depth)
  const tagName = FiberTagMap[fiber.tag] || `Unknown(${fiber.tag})`

  let typeName: string
  if (typeof fiber.type === 'function') typeName = fiber.type.name || 'anonymous'
  else if (typeof fiber.type === 'string') typeName = fiber.type
  else if (fiber.type === null) typeName = 'null'
  else typeName = String(fiber.type)

  const hooksInfo = formatHooks(fiber.memoizedState)
  const altInfo = fiber.alternate ? ' ⟲hasAlternate' : ''
  const domInfo = fiber.stateNode instanceof HTMLElement
    ? `  dom: <${fiber.stateNode.tagName.toLowerCase()}>` : ''

  lines.push(`${indent}${tagName} <${typeName}>${hooksInfo}${altInfo}${domInfo}`)

  if (fiber.child) traverseFiber(fiber.child, depth + 1, lines)
  if (fiber.sibling) traverseFiber(fiber.sibling, depth, lines)
}

// 子组件，让 Fiber 树有结构
function Counter() {
  const [count, setCount] = useState(0)
  return <button onClick={() => setCount(c => c + 1)}>count: {count}</button>
}

function Timer() {
  const [seconds, setSeconds] = useState(0)
  useEffect(() => {
    const id = setInterval(() => setSeconds(s => s + 1), 1000)
    return () => clearInterval(id)
  }, [])
  return <p>seconds: {seconds}</p>
}

export default function Task1FiberTraversal() {
  const [output, setOutput] = useState<string[]>([])

  const handleTraverse = () => {
    const rootEl = document.getElementById('root') as HTMLElement
    const fiberRoot = getFiberRoot(rootEl)

    const lines: string[] = []
    lines.push('=== Fiber 树遍历结果 ===')
    lines.push("入口: document.getElementById('root').__reactContainer$<key>")
    lines.push('获取: fiberRoot.current → HostRoot Fiber (tag=3)')
    lines.push('')

    if (!fiberRoot) {
      lines.push('未找到 Fiber 根节点')
    } else {
      traverseFiber(fiberRoot, 0, lines)
    }

    // 选做：探索 __CLIENT_INTERNALS
    lines.push('')
    lines.push('=== __CLIENT_INTERNALS 探索（选做）===')
    const internals = (ReactNS as any).__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE
    if (internals) {
      lines.push(`keys: ${Object.keys(internals).join(', ')}`)
      lines.push('包含 Hooks 调度器等运行时状态，不直接包含 Fiber 树')
      lines.push('React 18 旧名: __SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED')
    } else {
      lines.push('未找到（可能需要 development 模式）')
    }

    setOutput(lines)
    console.log(lines.join('\n'))
  }

  return (
    <div>
      <p>页面中渲染了 Counter 和 Timer 组件，点击按钮遍历它们的 Fiber 树：</p>
      <div style={{ padding: 12, border: '1px solid #ddd', borderRadius: 8, marginBottom: 12 }}>
        <Counter /> <Timer />
      </div>
      <button onClick={handleTraverse}>遍历 Fiber 树</button>
      {output.length > 0 && (
        <pre style={{
          background: '#1e1e1e', color: '#d4d4d4',
          padding: 16, borderRadius: 8, overflow: 'auto',
          fontSize: 13, lineHeight: 1.6, maxHeight: 500,
        }}>
          {output.join('\n')}
        </pre>
      )}
    </div>
  )
}
