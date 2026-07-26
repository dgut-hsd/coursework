# Day 1：React 19 内核 — Fiber、Hooks 链表与闭包陷阱

> 项目用 React 19.2。今天你不学"怎么用 useState"——你理解 Fiber 协调器怎么调度渲染、Hooks 链表怎么存在 Fiber 节点上、闭包陷阱为什么发生、React 19 的 `use()` 和 `ref` as prop 带来了什么。

---

## 学习目标

1. 理解 Fiber reconciler 的节点结构和双缓冲机制
2. 从 Fiber.memoizedState 理解 Hooks 链表和闭包陷阱的根因
3. 掌握 React 19 新增 `use()` / ref as prop / `useOptimistic` / 异步清理
4. 了解 React 19 破坏性变化（`propTypes` 移除、`defaultProps` 废弃、`<Context>` 直接替代 `<Context.Provider>`）

---

## 核心概念

### 1. Fiber — React 的"虚拟 DOM"其实是一棵树链表

React 19 的 Fiber reconciler 与 React 18 在核心机制上一致——可中断的异步渲染、优先级调度、双缓冲。

```
Fiber 节点结构（关键字段）：

  child    → 第一个子节点
  sibling  → 下一个兄弟节点（树转链表）
  return   → 父节点
  alternate → current ↔ workInProgress 双缓冲指针

  memoizedState → Hooks 链表头
  memoizedProps → 上次渲染的 props
  stateNode     → DOM 节点引用

  lanes     → 31 位优先级位图
  childLanes → 子树中最高优先级
```

React 19 的变化：改进了并发渲染的调度稳定性、减少了未使用状态造成的额外渲染、`use()` 允许在渲染中直接读取异步数据而无需 Suspense 边界。

---

### 2. Hooks 链表与闭包陷阱

Hooks 在 Fiber.memoizedState 上是**单向链表**：

```
fiber.memoizedState → {
  memoizedState: 0,           // useState(0)
  queue: { dispatch: setState },
  next: {
    memoizedState: {          // useEffect(cb, deps)
      create: callback,
      destroy: cleanup,
      deps: [0]
    },
    next: null
  }
}
```

闭包陷阱的根因不变：effect 回调创建时捕获了当时的闭包变量。如果 deps 为空，回调永远用第一次的值。

```typescript
// ❌ 陷阱：count 永远是 0
useEffect(() => {
  const id = setInterval(() => setCount(count + 1), 1000);
  return () => clearInterval(id);
}, []);

// ✅ 解法 1：函数式更新
setCount(c => c + 1);

// ✅ 解法 2：正确 deps
useEffect(() => { ... }, [count]);
```

---

### 3. React 19 新特性（对标项目）

#### `use()` — 在渲染中读数据

```typescript
// React 19：直接在组件中用 use() 读取 Promise
function AuditTask({ taskId }: { taskId: string }) {
  const task = use(fetchTask(taskId));  // 不阻塞兄弟组件
  return <TaskDetail task={task} />;
}

// 替代了"useState + useEffect + if (loading)"的三段式
```

#### ref 作为 prop — 不再需要 forwardRef

```typescript
// React 18：需要 forwardRef 包装
const MyInput = forwardRef<HTMLInputElement, Props>((props, ref) => (
  <input ref={ref} {...props} />
));

// React 19：直接传 ref prop
function MyInput({ ref, ...props }: Props & { ref: Ref<HTMLInputElement> }) {
  return <input ref={ref} {...props} />;
}
```

本项目 antd 5.27 在 React 19 下已支持此用法。

#### useOptimistic — 乐观更新

```typescript
// 提交审核任务后立即显示"处理中"，不等服务器确认
const [optimisticTasks, addOptimistic] = useOptimistic(
  tasks,
  (state, newTask: AuditTask) => [...state, { ...newTask, status: 'PENDING' }]
);

function handleCreate(dto: CreateTaskDTO) {
  addOptimistic({ ...dto, id: tempId });
  // 实际请求在后台进行——失败时 useOptimistic 自动回滚
  createTaskMutation.mutate(dto);
}
```

#### 异步清理函数

```typescript
// React 19：useEffect 的 cleanup 可以返回 Promise
useEffect(() => {
  const es = new EventSource('/api/stream');
  es.onmessage = handleMessage;
  return async () => {
    es.close();
    await persistState();  // 关闭连接前保存状态
  };
}, []);
```

---

## 动手

### 任务 1：Fiber 树遍历

**入口说明**：React 在两处暴露了内部调试入口——

| 入口 | 来源 | 能拿到什么 |
|------|------|-----------|
| DOM 元素上的 `__reactContainer$` | `react-dom` 渲染时挂在容器 DOM 上 | Fiber 树根节点（`stateNode.current`） |
| `__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE`（React 18 旧名 `__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED`） | `react` 模块导出 | Hooks 调度器、渲染内部状态 |

> **注意**：React 19 中 `__SECRET_INTERNALS` 已重命名为 `__CLIENT_INTERNALS`，且其内容为 Hooks 调度器等运行时状态，**不直接包含 Fiber 树**。遍历 Fiber 树请走 DOM 入口。

**要求**：通过 DOM 元素上的 `__reactContainer$` → `stateNode.current` 拿到 Fiber 树根节点，递归遍历打印关键字段（`tag`、`type`、`child`/`sibling`/`return` 指针、`memoizedState` Hooks 链表、`alternate` 双缓冲指针、`stateNode` DOM 引用）。对比 DevTools 验证。

**额外探索**（选做）：在 `main.tsx` 中引入 `__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE` 挂到 `window` 上，在控制台观察 `H`（Hooks 调度器）在渲染期与非渲染期的差异。

### 任务 2：闭包陷阱实验室

复现 3 个经典陷阱——`setInterval` 读到旧值、`useEffect` deps 为空导致状态过期、`useCallback` 引用不稳定。每种提供修复方案。

### 任务 3：React 19 新特性对比

分别用 `use()` + `useOptimistic()` 重写一个"提交审核任务"的组件。对比与 React 18 写法的代码量差异。

---

## 验收标准

- [ ] Fiber 遍历器正确输出树结构（通过 `__reactContainer$` DOM 入口）
- [ ] 理解 `__CLIENT_INTERNALS` 与 Fiber 树的区别（一个给 Hooks 调度器，一个给树结构）
- [ ] 3 个闭包陷阱全部修复 + 根因分析
- [ ] React 19 新特性示例可运行

---

## 思考题

1. `use()` 和 `useEffect + useState` 的本质区别是什么？什么场景不适合用 `use()`？
2. React 19 的 `ref` as prop 消除了 `forwardRef`——但 Custom Component 的 `ref` 传递给哪个 DOM 节点是隐式的。怎么让使用者知道 ref 挂在了哪个元素上？
3. `useOptimistic` 在失败时自动回滚——但如果乐观更新时间窗口内用户做了其他操作（如删除了同一条任务），回滚逻辑怎么处理？

---

## 与标书审核项目的关系

审核工作台的 SSE 实时流（Day 3）可以在连接关闭时使用 React 19 的异步清理函数保存状态。乐观更新（`useOptimistic`）让创建审核任务时用户体验更流畅——点击"开始审核"后立即显示进度条，不等待 HTTP 响应。
