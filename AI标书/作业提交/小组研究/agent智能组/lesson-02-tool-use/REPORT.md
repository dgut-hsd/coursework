# Lesson 02: Tool Use + ToolRegistry — 报告

## 一、项目结构

```
lesson-02-tool-use/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── tool_registry.rs     # AgentTool trait + ToolRegistry + 3个工具实现
│   ├── search_buffer.rs     # SearchBuffer 并发去重
│   └── error_isolation.rs   # 工具执行隔离（错误/超时/重试）
└── REPORT.md
```

## 二、ToolRegistry 架构

```
┌─────────────────────────────────────────────────────────────┐
│                     ToolRegistry                            │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  HashMap<String, Box<dyn AgentTool>>                   │  │
│  │                                                       │  │
│  │  ┌─────────────────┐  ┌─────────────────┐             │  │
│  │  │ search_knowledge│  │ read_section    │             │  │
│  │  │ (SearchKnowledgeTool)  (ReadSectionTool)           │  │
│  │  └─────────────────┘  └─────────────────┘             │  │
│  │                                                       │  │
│  │  ┌─────────────────┐  ┌─────────────────┐             │  │
│  │  │ output_finding  │  │ flaky_search    │             │  │
│  │  │ (OutputFindingTool)   (FlakySearchTool)            │  │
│  │  └─────────────────┘  └─────────────────┘             │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  register(tool) → 插入 HashMap                              │
│  definitions() → 返回工具定义列表（给 LLM）                   │
│  execute(name, args) → 执行指定工具                          │
└─────────────────────────────────────────────────────────────┘
```

## 三、SearchBuffer 并发去重机制

```
Agent A ─────┐
             │  query="建筑工程资质"
Agent B ─────┼──→ SearchBuffer.search()
             │
Agent C ─────┘

                 │
                 ▼
         ┌───────────────┐
         │ 查缓存        │
         │ cache.get(key)│
         └───────┬───────┘
                 │ 未命中
                 ▼
         ┌───────────────┐
         │ 查 pending    │
         │ pending.get() │
         └───────┬───────┘
                 │ 无进行中
                 ▼
         ┌───────────────┐
         │ 启动新搜索     │
         │ tokio::spawn()│
         │ → JoinHandle  │
         │ → 插入 pending│
         └───────┬───────┘
                 │
                 ▼
         ┌───────────────┐
         │ 等待结果      │
         │ handle.await  │
         └───────┬───────┘
                 │
                 ▼
         ┌───────────────┐
         │ 写入缓存      │
         │ 更新 pending  │
         └───────┬───────┘
                 │
                 ▼
        返回结果给所有3个Agent
```

## 四、错误隔离机制

```rust
async fn execute_tool_safe(tool: &dyn AgentTool, args: &Value, timeout: Duration) -> ToolResult {
    match timeout(timeout, tool.execute(args.clone())).await {
        Ok(Ok(result)) => ToolResult::Success(result),      // 成功
        Ok(Err(e)) => ToolResult::Error(format!("{}", e)),  // 错误隔离
        Err(_) => ToolResult::Timeout(tool.name().to_string()), // 超时隔离
    }
}
```

## 五、关键决策点

### 决策点 1: AgentTool trait 的设计

**位置**: `src/tool_registry.rs:5-9`

```rust
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn definition(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<Value, ToolError>;
}
```

**设计理由**: 这是 G4（Agent 侧）和 G5（工具实现侧）的解耦接口。G5 实现工具，G4 通过 `definitions()` 获取工具列表发给 LLM，两组可并行开发。

### 决策点 2: SearchBuffer 的 pending 设计

**位置**: `src/search_buffer.rs:59-91`

```rust
pub async fn search(&self, query: &str, searcher: &dyn Searcher) -> SearchResult {
    // 先查缓存 → 再查 pending → 最后启动新搜索
    // pending 存 JoinHandle，后续请求 await 同一个 Future
}
```

**设计理由**: 避免多个 Agent 同时搜索相同关键词时发出多次 HTTP 请求，减少 API 成本和响应时间。

### 决策点 3: 错误不传播到 Agent 循环

**位置**: `src/error_isolation.rs:10-18`

```rust
// 三个关键隔离：
// 1. 错误隔离：工具抛错 → 捕获为 ToolResult::Error
// 2. 超时隔离：timeout 5秒 → 返回 ToolResult::Timeout
// 3. 重试策略：失败不重试工具本身，而是把错误注入对话 → Agent 自己决定是否重试
```

**设计理由**: 如果工具错误直接传播，会导致整个 Agent 循环崩溃。捕获错误后注入对话，Agent 可以根据错误信息决定换关键词重试。

## 六、验收标准验证

| 验收项 | 状态 | 验证方法 |
|--------|------|----------|
| ToolRegistry 正确注入 3 个工具 | ✅ | `test_tool_registry_register`, `test_tool_registry_definitions` |
| SearchBuffer：3 并发 → 1 次 HTTP 请求 | ✅ | `test_search_buffer_concurrent_deduplication` |
| 工具错误不传播到 Agent 循环 | ✅ | `test_error_isolation_does_not_crash` |

## 七、测试结果

```
running 15 tests
test tool_registry::tests::test_tool_registry_register ... ok
test tool_registry::tests::test_tool_registry_definitions ... ok
test tool_registry::tests::test_search_knowledge_execute ... ok
test tool_registry::tests::test_read_section_execute ... ok
test tool_registry::tests::test_output_finding_execute ... ok
test search_buffer::tests::test_search_buffer_single_request ... ok
test search_buffer::tests::test_search_buffer_concurrent_deduplication ... ok
test search_buffer::tests::test_search_buffer_caching ... ok
test search_buffer::tests::test_search_buffer_different_queries ... ok
test search_buffer::tests::test_search_buffer_normalization ... ok
test error_isolation::tests::test_execute_tool_safe_success ... ok
test error_isolation::tests::test_execute_tool_safe_timeout ... ok
test error_isolation::tests::test_execute_tool_safe_error ... ok
test error_isolation::tests::test_execute_tool_with_retry_success_after_retry ... ok
test error_isolation::tests::test_error_isolation_does_not_crash ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 八、设计决策文档

### 1. 为什么用 trait 而不是函数指针？

**问题**: 工具应该如何抽象？

**方案**: 使用 `AgentTool` trait

**对比替代方案**:
- **函数指针**: 无法携带状态，难以实现复杂工具
- **闭包**: 无法在 trait 对象中使用
- **枚举**: 每次新增工具都要修改枚举定义

**选择理由**: trait 提供了最大的灵活性，每个工具可以有自己的状态和实现。

### 2. SearchBuffer 的并发安全性

**问题**: 多个 Agent 同时搜索时如何保证数据一致性？

**方案**: `Arc<RwLock<HashMap>>` + `Arc<RwLock<LruCache>>`

**对比替代方案**:
- **Mutex**: 写操作时完全阻塞，并发性能差
- **无锁数据结构**: 实现复杂，容易出错
- **全局唯一缓存**: 难以管理生命周期

**选择理由**: RwLock 在多读少写场景下性能优于 Mutex，且实现相对简单。

### 3. 为什么工具错误不重试？

**问题**: 工具执行失败时是否应该自动重试？

**方案**: 不自动重试，把错误注入对话让 Agent 决定

**对比替代方案**:
- **自动重试**: 可能陷入无限重试循环
- **固定重试次数**: 无法处理所有情况
- **指数退避**: 增加延迟

**选择理由**: Agent 能根据错误类型（网络错误 vs 业务错误）做出更智能的决策。

### 4. 局限性

- **SearchBuffer 的 pending 清理**: 如果搜索任务 panic，JoinHandle 可能永远留在 pending 中
- **错误信息质量**: 工具返回的错误信息需要足够详细，Agent 才能正确决策
- **重试策略**: 当前实现没有区分工具类型，所有工具都使用相同的重试策略
