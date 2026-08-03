import { useState, Suspense, lazy } from 'react'
import { Card, Button, Spin, Typography, List } from 'antd'

const { Text, Paragraph } = Typography

// React.lazy 代码分割：LazyChunk（含 echarts）被拆成独立 chunk
const LazyEchartsChunk = lazy(() => import('./LazyChunk'))

const bundleConfigCode = `// vite.config.ts
import { visualizer } from 'rollup-plugin-visualizer'

export default defineConfig({
  plugins: [
    visualizer({ open: true, gzipSize: true }),
  ],
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          echarts: ['echarts'],  // ~1MB → 独立 chunk
          react: ['react', 'react-dom'],
        },
      },
    },
  },
})`

const lazyCode = `// React.lazy：echarts 只在需要时加载
const LazyEchartsChunk = lazy(() => import('./LazyChunk'))

// 使用时用 Suspense 包裹
<Suspense fallback={<Spin />}>
  <LazyEchartsChunk />
</Suspense>`

export default function Task3BundleAnalysis() {
  const [showLazy, setShowLazy] = useState(false)

  return (
    <div>
      <Card title="Bundle 分析配置" size="small" style={{ marginBottom: 16 }}>
        <pre style={{ fontSize: 12, overflow: 'auto' }}>{bundleConfigCode}</pre>
        <Text type="secondary">运行 npm run build → 自动打开 bundle-stats.html 查看各包大小</Text>
      </Card>

      <Card title="React.lazy 代码分割" size="small" style={{ marginBottom: 16 }}>
        <pre style={{ fontSize: 12, overflow: 'auto' }}>{lazyCode}</pre>
        <Paragraph>
          点击下方按钮加载 LazyChunk（含 echarts）。首次加载时会下载独立 chunk。
        </Paragraph>
        <Button
          type="primary"
          onClick={() => setShowLazy(true)}
          disabled={showLazy}
        >
          {showLazy ? '已加载' : '加载 LazyChunk (echarts)'}
        </Button>
        {showLazy && (
          <Suspense fallback={<Spin style={{ display: 'block', margin: '16px auto' }} />}>
            <div style={{ marginTop: 16 }}>
              <LazyEchartsChunk />
            </div>
          </Suspense>
        )}
      </Card>

      <Card title="Lighthouse 审计目标" size="small">
        <List
          size="small"
          dataSource={[
            { metric: 'FCP', target: '< 1.8s' },
            { metric: 'LCP', target: '< 2.5s' },
            { metric: 'TBT', target: '< 200ms' },
            { metric: 'Performance Score', target: '> 90' },
          ]}
          renderItem={(item) => (
            <List.Item>
              <Text strong>{item.metric}</Text>: {item.target}
            </List.Item>
          )}
        />
        <Text type="secondary" style={{ fontSize: 13 }}>
          Chrome DevTools → Lighthouse → Performance 审计
        </Text>
      </Card>
    </div>
  )
}
