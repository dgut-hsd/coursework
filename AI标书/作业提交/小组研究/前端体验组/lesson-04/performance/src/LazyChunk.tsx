import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

// 这个组件被 React.lazy 动态导入 → echarts 被拆成独立 chunk
export default function LazyChunk() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)
    chart.setOption({
      title: { text: 'echarts 独立 chunk 示例', left: 'center' },
      tooltip: { trigger: 'item' },
      series: [{
        type: 'pie',
        radius: ['40%', '70%'],
        data: [
          { value: 3, name: '严重', itemStyle: { color: '#cf1322' } },
          { value: 5, name: '警告', itemStyle: { color: '#d48806' } },
          { value: 8, name: '信息', itemStyle: { color: '#1677ff' } },
        ],
      }],
    })
    return () => chart.dispose()
  }, [])

  return (
    <div>
      <p>此组件 + echarts (~1MB) 通过 React.lazy 延迟加载，不会阻塞首屏</p>
      <div ref={ref} style={{ width: '100%', height: 250 }} />
    </div>
  )
}
