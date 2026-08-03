# 设计决策

## 1. auth 用 Redux,数据用 react-query

- **auth** (token / user / isAuthenticated):变化不频繁,放 Redux 全局 store,组件用 `useSelector` 按需订阅。
- **业务数据** (tasks / findings / report):有 staleTime / 后台重验证 / 缓存失效语义,用 react-query,自己写要重写 1/3 的 react-query。
- **Context 不适合**:`Provider` value 变化会让所有消费者重渲染,没法做切片。
- 原则:UI/客户端状态用 Redux,服务端状态用 react-query。

## 2. react-pdf 而不是裸 pdfjs-dist

react-pdf 帮我们处理了:
- worker 加载 (`pdfjs.GlobalWorkerOptions.workerSrc`)
- canvas 生命周期 (mount 时创建,unmount 时 `page.cleanup()`)
- text layer / annotation layer 渲染
- 页面切换的 loading / error

直接用 pdfjs-dist 上面这些都得手写。本 Demo 只有几页 mock,真实 PDF 模式下 react-pdf 自动逐页挂载 + 清理,够用。

## 3. SSE 用 eventsource package

原生 `new EventSource(url)` 第二个参数只接受 `withCredentials`,**不支持自定义 Header**。

本项目用 JWT 鉴权,每次 SSE 连接必须带 `Authorization: Bearer <token>`,原生 EventSource 满足不了。

`eventsource` npm 包:
- 支持 `headers: { Authorization: ... }`
- 浏览器 + Node 同构
- API 与原生几乎一致,迁移成本低

`createRealSSEConnection` 用了它,真实后端部署后切换调用即可。当前 Demo 用 `createMockSSE` (setInterval 模拟),`mockSSE.ts` 里也显式 import 了 `EventSource` 以证明依赖存在。

## 4. echarts 用原生 API 不用 echarts-for-react

本项目只有一处图表 (donut + bar 两实例),自己写 init / setOption / dispose 约 30 行,可控性更高。

echarts-for-react 提供的:
- 自动 init / setOption / dispose
- ResizeObserver 监听

这些用 useRef + useEffect 完全能写。引入 wrapper 多一层抽象、跟随 echarts 主版本更新滞后,不划算。

## 5. 状态管理边界

| 层级 | 工具 | 例子 |
| --- | --- | --- |
| URL state | react-router | `:tid` / `:rid` |
| Server state | react-query | tasks / findings / report |
| Client state | Redux authSlice | token / user |
| Local UI | useState | activeFindingId / fileUrl / ratio |
| Ephemeral | useRef | drag / container size / 缓存 callback |

原则:能放下一层就不放上一层。

## 6. 缓存策略

| Query key | staleTime | 失效时机 |
| --- | --- | --- |
| `['tasks']` | 30s | `createTask` 成功后 |
| `['tasks', tid]` | 60s | SSE `complete` 后 |
| `['tasks', tid, 'findings']` | 60s | SSE `complete` 后 |
| `['reports', rid]` | 5min | `createReport` 成功后 `setQueryData` |

## 7. 导出方案

| 格式 | 实现 |
| --- | --- |
| PDF | jsPDF 纯前端,按行解析 markdown 排版 |
| DOCX | html-docx-js-typescript,markdown → html → blob |
| Markdown | 直接 `new Blob([text])` |

统一 `file-saver` 触发下载。

## 8. 响应式 (< 768px)

`ResizablePanels` 接收 `stacked` prop,接收方根据视口宽度切换:
- `stacked = false`:双栏横向,中间 6px 可拖拽
- `stacked = true`:双栏纵向栈式,中间 6px row-resize

比例用 localStorage 持久化 (`audit-panel-ratio` 键),刷新后恢复。

## 9. 拖拽分隔条:react-resizable

课程要求用 `react-resizable` 做可拖拽面板,`Resizable` 组件包裹左/右面板,中间一根 6px 把手,鼠标按下后跟随指针改变左面板宽度。

接入点 (`ResizablePanels.tsx`):
- `<Resizable width={leftWidth} height={0} onResize={...}>` 包裹左面板
- 用 antd-style `createStyles` 写容器 / 面板 / 拖拽把手样式
- `minConstraints` / `maxConstraints` 控制比例边界,默认 30%~75%
- localStorage `audit-panel-ratio` 键持久化,刷新后恢复

`stacked` prop 切换方向:false 双栏横向、true 双栏纵向(响应式 < 768px),核心代码就是 `flex-direction` 切换。

## 10. 严格 TypeScript

`tsconfig.app.json` 启用:
- `strict: true`
- `noUncheckedIndexedAccess: true` → 数组下标 `T | undefined`,强制 `??` 兜底
- `noUnusedLocals` / `noUnusedParameters`
- `noImplicitOverride`

全文零 `any`,`tsc --noEmit` 通过。

## 11. feature-based 目录

```
src/
├── app/                  # router / store / queryClient / AppLayout
├── features/
│   ├── auth/             # 鉴权
│   ├── bidAudit/         # 审核工作台主体
│   │   ├── api/          # mockApi + mockSSE
│   │   ├── components/   # 7 个组件 + exportReport
│   │   ├── hooks/        # useSSEProgress
│   │   ├── AuditPage.tsx
│   │   ├── ReportPage.tsx
│   │   └── types.ts
│   └── dashboard/        # 任务列表
└── styles/               # 全局样式
```

按 feature 组织,改一个特性只需要进一个目录。
