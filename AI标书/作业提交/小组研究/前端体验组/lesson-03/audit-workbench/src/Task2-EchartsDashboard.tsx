import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'
import { useDebounceFn } from 'ahooks'

export default function Task2EchartsDashboard() {
  const donutRef = useRef<HTMLDivElement>(null)
  const barRef = useRef<HTMLDivElement>(null)
  const donutChart = useRef<echarts.ECharts | null>(null)
  const barChart = useRef<echarts.ECharts | null>(null)

  // 环形图
  useEffect(() => {
    if (!donutRef.current) return
    donutChart.current = echarts.init(donutRef.current)
    donutChart.current.setOption({
      title: { text: '问题严重性分布', left: 'center' },
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
    return () => donutChart.current?.dispose()
  }, [])

  // 柱状图
  useEffect(() => {
    if (!barRef.current) return
    barChart.current = echarts.init(barRef.current)
    barChart.current.setOption({
      title: { text: '各 Agent 发现数量', left: 'center' },
      tooltip: { trigger: 'axis' },
      xAxis: { type: 'category', data: ['资质审查', '报价审核', '技术方案', '合规检查'] },
      yAxis: { type: 'value' },
      series: [{ type: 'bar', data: [5, 3, 4, 2], itemStyle: { color: '#1677ff' } }],
    })
    return () => barChart.current?.dispose()
  }, [])

  // resize 防抖（ahooks useDebounceFn）
  const { run: handleResize } = useDebounceFn(() => {
    donutChart.current?.resize()
    barChart.current?.resize()
  }, { wait: 200 })

  useEffect(() => {
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [handleResize])

  return (
    <div>
      <p>环形图 + 柱状图，缩放窗口验证 resize 防抖</p>
      <div ref={donutRef} style={{ width: '100%', height: 300 }} />
      <div ref={barRef} style={{ width: '100%', height: 300, marginTop: 16 }} />
    </div>
  )
}
