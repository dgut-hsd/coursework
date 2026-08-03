import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { visualizer } from 'rollup-plugin-visualizer'

export default defineConfig({
  plugins: [
    react(),
    // Bundle 分析：运行 npm run build 后自动打开 bundle-stats.html
    visualizer({
      open: true,
      gzipSize: true,
      filename: 'bundle-stats.html',
    }),
  ],
  build: {
    rollupOptions: {
      output: {
        // 演示代码分割：echarts 等大依赖被分到独立 chunk
        manualChunks: {
          echarts: ['echarts'],
          react: ['react', 'react-dom'],
        },
      },
    },
  },
})
