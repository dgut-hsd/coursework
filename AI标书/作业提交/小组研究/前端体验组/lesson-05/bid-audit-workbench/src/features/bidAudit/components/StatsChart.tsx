import { useEffect, useRef } from 'react';
import type { JSX } from 'react';
import * as echarts from 'echarts/core';
import { PieChart, BarChart } from 'echarts/charts';
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
} from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import type { Finding, Severity } from '../types';

echarts.use([
  PieChart,
  BarChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  CanvasRenderer,
]);

interface StatsChartProps {
  findings: Finding[];
}

const SEVERITY_COLORS: Record<Severity, string> = {
  critical: '#ff4d4f',
  warning: '#faad14',
  info: '#1677ff',
};

const SEVERITY_LABELS: Record<Severity, string> = {
  critical: '严重',
  warning: '警告',
  info: '提示',
};

export function StatsChart({ findings }: StatsChartProps): JSX.Element {
  const donutRef = useRef<HTMLDivElement | null>(null);
  const barRef = useRef<HTMLDivElement | null>(null);
  const donutChartRef = useRef<echarts.ECharts | null>(null);
  const barChartRef = useRef<echarts.ECharts | null>(null);

  useEffect(() => {
    if (donutRef.current && !donutChartRef.current) {
      donutChartRef.current = echarts.init(donutRef.current);
    }
    if (barRef.current && !barChartRef.current) {
      barChartRef.current = echarts.init(barRef.current);
    }
    const onResize = (): void => {
      donutChartRef.current?.resize();
      barChartRef.current?.resize();
    };
    window.addEventListener('resize', onResize);
    return () => {
      window.removeEventListener('resize', onResize);
    };
  }, []);

  useEffect(() => {
    return () => {
      donutChartRef.current?.dispose();
      barChartRef.current?.dispose();
      donutChartRef.current = null;
      barChartRef.current = null;
    };
  }, []);

  useEffect(() => {
    const counts: Record<Severity, number> = { critical: 0, warning: 0, info: 0 };
    for (const f of findings) {
      counts[f.severity] += 1;
    }

    const donutOption: echarts.EChartsCoreOption = {
      tooltip: {
        trigger: 'item',
        formatter: '{b}: {c} ({d}%)',
      },
      legend: {
        bottom: 0,
        icon: 'circle',
        textStyle: { fontSize: 12 },
      },
      series: [
        {
          name: '问题分布',
          type: 'pie',
          radius: ['45%', '70%'],
          center: ['50%', '45%'],
          avoidLabelOverlap: false,
          itemStyle: {
            borderRadius: 4,
            borderColor: '#fff',
            borderWidth: 2,
          },
          label: {
            show: true,
            formatter: '{b}\n{c}',
            fontSize: 12,
          },
          labelLine: { show: true },
          data: (['critical', 'warning', 'info'] as Severity[]).map((s) => ({
            name: SEVERITY_LABELS[s],
            value: counts[s],
            itemStyle: { color: SEVERITY_COLORS[s] },
          })),
        },
      ],
    };

    const pageCounts: Record<number, number> = {};
    for (const f of findings) {
      pageCounts[f.page] = (pageCounts[f.page] ?? 0) + 1;
    }
    const pageEntries = Object.entries(pageCounts)
      .map(([p, c]) => ({ page: Number(p), count: c }))
      .sort((a, b) => a.page - b.page);

    const barOption: echarts.EChartsCoreOption = {
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
      },
      grid: { top: 24, right: 16, bottom: 32, left: 32, containLabel: true },
      xAxis: {
        type: 'category',
        data: pageEntries.map((e) => `第 ${e.page} 页`),
        axisTick: { alignWithLabel: true },
        axisLabel: { fontSize: 11 },
      },
      yAxis: {
        type: 'value',
        minInterval: 1,
        axisLabel: { fontSize: 11 },
      },
      series: [
        {
          name: '问题数',
          type: 'bar',
          data: pageEntries.map((e) => e.count),
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#1677ff' },
              { offset: 1, color: '#4096ff' },
            ]),
            borderRadius: [4, 4, 0, 0],
          },
          barWidth: '40%',
          label: { show: true, position: 'top', fontSize: 11 },
        },
      ],
    };

    donutChartRef.current?.setOption(donutOption, true);
    barChartRef.current?.setOption(barOption, true);
  }, [findings]);

  return (
    <div>
      <div ref={donutRef} style={{ width: '100%', height: 220 }} />
      <div ref={barRef} style={{ width: '100%', height: 200 }} />
    </div>
  );
}
