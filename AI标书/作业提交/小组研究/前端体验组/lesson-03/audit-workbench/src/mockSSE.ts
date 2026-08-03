// Mock SSE — 模拟审核过程的事件推送
// 实际项目中用 new EventSource(url) 连接服务器

export interface SSEMessage {
  type: 'progress' | 'finding' | 'complete'
  data: any
}

const mockReport = `# 标书审核报告

## 审核结果总览

| 审核项 | 结果 |
|--------|------|
| 资质审查 | 不通过 |
| 报价审核 | 不通过 |
| 技术方案 | 通过 |
| 合规检查 | 通过 |

## 发现的问题

### 1. 严重：投标报价超过限价

报价金额超出《招标投标法》规定的最高限价 15%。

依据：《中华人民共和国招标投标法》第三十三条。

### 2. 警告：缺少资质证书

未附安全生产许可证，不符合《安全生产许可证条例》要求。

依据：《安全生产许可证条例》第六条。
`

export function startMockSSE(onMessage: (msg: SSEMessage) => void): () => void {
  let cancelled = false

  const stages = [
    { name: '文档解析', progress: 15 },
    { name: '资质审查', progress: 30 },
    { name: '报价审核', progress: 55 },
    { name: '技术方案', progress: 75 },
    { name: '合规检查', progress: 90 },
    { name: '生成报告', progress: 100 },
  ]

  const findings = [
    { id: 'f1', severity: 'critical', title: '投标报价超过限价', description: '报价金额超出最高限价 15%' },
    { id: 'f2', severity: 'warning', title: '缺少资质证书', description: '未附安全生产许可证' },
  ]

  let i = 0

  function emit() {
    if (cancelled) return
    const stage = stages[i]
    onMessage({ type: 'progress', data: { progress: stage.progress, currentStage: stage.name } })

    if (i === 1) onMessage({ type: 'finding', data: findings[0] })
    if (i === 3) onMessage({ type: 'finding', data: findings[1] })

    i++
    if (i < stages.length) {
      setTimeout(emit, 1000)
    } else {
      setTimeout(() => { if (!cancelled) onMessage({ type: 'complete', data: { report: mockReport } }) }, 500)
    }
  }

  setTimeout(emit, 500)
  return () => { cancelled = true }
}
