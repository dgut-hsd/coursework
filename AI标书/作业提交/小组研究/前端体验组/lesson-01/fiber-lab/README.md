# Fiber Lab — React 19 Fiber 树遍历实验

> Day 1 任务 1：通过 React 内部 API 遍历 Fiber 树，观察 Hooks 链表与双缓冲机制

## 快速复现

```bash
cd fiber-lab
pnpm install
pnpm dev
```

浏览器打开 http://localhost:5173，按 F12 打开控制台，粘贴实验报告中的遍历脚本即可看到 Fiber 树结构。

## 文件说明

| 路径 | 内容 |
|------|------|
| `src/App.tsx` | 根组件（useState） |
| `src/TaskCard.tsx` | 子组件（5 个 Hook：useState×2 + useEffect + useCallback×2） |
| `src/main.tsx` | 入口，暴露 `window.__REACT_INTERNALS` |
| `实验报告/实验报告.md` | 完整实验报告（步骤 + 分析 + 心得） |
