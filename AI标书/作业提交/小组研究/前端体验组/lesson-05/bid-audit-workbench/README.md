# 标书审核工作台 Demo

React 19 + TypeScript strict 的 feature-based 审核工作台。

## 运行

```bash
npm install
npm run dev          # 启动 dev server
npm run typecheck    # tsc 严格类型检查
npm run build        # 生产构建
```

登录页任意邮箱密码均可进入。点击 "开始审核" 进入工作台,等待约 6 秒 SSE 进度走完,左侧 PDF 自动出现高亮框,右侧问题列表依次浮现。

## 功能点

- 文件上传 (antd Upload.Dragger)
- PDF 预览 (react-pdf) + bbox 高亮 overlay
- 可拖拽双栏 (react-resizable,localStorage 持久化比例)
- SSE 实时进度 (eventsource package)
- ECharts 统计 (环形图 + 柱状图)
- 审核报告渲染 (react-markdown + remark-gfm)
- 报告导出 (PDF / DOCX / Markdown,file-saver 触发下载)
- Redux authSlice + react-query 任务缓存
- < 768px 单栏栈式

## 目录

```
src/
├── app/                  # router / store / queryClient
├── features/
│   ├── auth/             # Redux authSlice + LoginPage
│   ├── bidAudit/         # 审核工作台主体
│   │   ├── api/          # mockApi / mockSSE
│   │   ├── components/   # PdfViewer / BboxOverlay / ...
│   │   └── hooks/        # useSSEProgress
│   └── dashboard/        # 任务列表
└── styles/               # 全局样式
```

设计决策见 [DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md)。
