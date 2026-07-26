# Lesson 01: ReAct Loop 内核 — 报告

## 一、项目结构

```
lesson-01-react-loop/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── token_budget.rs      # Token Budget 管理器 + 上下文裁剪策略
│   ├── stream_parser.rs     # SSE 流式事件解析器
│   └── react_loop.rs        # ReAct Loop 核心逻辑
└── REPORT.md
```

## 二、消息流转时序图

```
User → System Prompt → User Query → LLM → Assistant(tool_calls) → Tool Execution → Tool Result → LLM → ... → output_finding

┌──────┐     ┌──────────────┐     ┌──────────┐     ┌────┐     ┌───────────────┐     ┌────────────┐     ┌───────────────┐     ┌────┐
│ User │────→│ System Prompt│────→│User Query│────→│ LLM│────→│ Assistant     │────→│ Tool       │────→│ Tool Result   │────→│ LLM│──→...──→output_finding
└──────┘     └──────────────┘     └──────────┘     └────┘     │ (tool_calls)  │     │ Execution  │     │               │     └────┘
                                                              └───────────────┘     └────────────┘     └───────────────┘
                                                                  │                          │                      │
                                                                  └───────────────────────────┴──────────────────────┘
                                                                                ↓
                                                              ┌──────────────────────────────┐
                                                              │      Token Budget 监控       │
                                                              │  (每轮检查剩余 token)         │
                                                              └──────────────────────────────┘
```

## 三、关键决策点

### 决策点 1: Token Budget 裁剪策略优先级

**位置**: `src/token_budget.rs:101-155`

```rust
// 裁剪策略优先级设计
fn trim_messages(&mut self) -> Result<(), BudgetError> {
    // 优先级 1: 保留 System Prompt（Agent 的角色定义不能丢）
    // 优先级 2: 保留最近 3 轮对话 + 工具调用结果
    // 优先级 3: 保留早期的工具调用结果（Agent 需要上下文）
    // 优先级 4: 丢弃最早的对话轮次
    // 优先级 5: 丢弃纯文本思考（中间推理可以丢弃）
}
```

**设计理由**: System Prompt 是 Agent 的核心指令，丢失会导致 Agent 行为失控。工具调用结果是 Agent 决策的基础数据，而中间思考文本只是过程记录，可以安全丢弃。

### 决策点 2: 流式 tool call 增量拼接

**位置**: `src/stream_parser.rs:127-168`

```rust
pub async fn parse_stream(...) -> Result<StreamOutput, StreamError> {
    let mut tool_calls: HashMap<usize, ToolCallBuilder> = HashMap::new();
    // ...
    if let Some(tc_deltas) = choice.delta.tool_calls {
        for tc in tc_deltas {
            let builder = tool_calls.entry(tc.index).or_default();
            if let Some(args) = tc.function.arguments {
                builder.arguments.push_str(&args);  // 增量拼接
            }
        }
    }
    // ...
    // 最后才反序列化完整 JSON
    let args: Value = serde_json::from_str(&b.arguments)?;
}
```

**设计理由**: `function.arguments` 是 JSON 字符串的增量片段，`"{\"qu` 无法解析，必须累积到完整的 `"{\"query\": \"建筑工程\"}"` 才能反序列化。

### 决策点 3: 三种停止条件的协作

**位置**: `src/react_loop.rs:38-68`

```rust
for turn in 0..config.max_turns {           // 停止条件 1: max_turns
    // ...
    if tc.function.name == "output_finding" { // 停止条件 2: 终端工具
        return Ok(messages);
    }
    // ...
    if budget_manager.remaining() < config.min_tokens_for_next_turn {
        break;                              // 停止条件 3: Token Budget 不足
    }
}
```

**设计理由**: 三层防护确保 Agent 不会无限循环或耗尽资源：`output_finding` 是正常终止，`max_turns` 防止无限工具调用，`Token Budget` 防止长文档审查时上下文溢出。

## 四、验收标准验证

| 验收项 | 状态 | 验证方法 |
|--------|------|----------|
| Token Budget 裁剪后 System Prompt 完整 | ✅ | `test_budget_manager_trim_preserves_system` |
| Token Budget 保留最近 3 轮消息 | ✅ | `test_budget_manager_trim_preserves_tool_results` |
| 流式解析正确拼接增量 tool call arguments | ✅ | `test_parse_tool_call_stream`, `test_parse_tool_call_incremental` |
| 时序图完整 | ✅ | 见本文档第二部分 |

## 五、测试结果

```
running 13 tests
test token_budget::tests::test_token_budget_basic ... ok
test token_budget::tests::test_budget_manager_add_messages ... ok
test token_budget::tests::test_budget_manager_trim_preserves_system ... ok
test token_budget::tests::test_budget_manager_trim_preserves_tool_results ... ok
test stream_parser::tests::test_parse_text_stream ... ok
test stream_parser::tests::test_parse_tool_call_stream ... ok
test stream_parser::tests::test_parse_tool_call_incremental ... ok
test stream_parser::tests::test_parse_error_frame ... ok
test stream_parser::tests::test_parse_ignores_non_data_lines ... ok
test stream_parser::tests::test_parse_mixed_text_and_tool_call ... ok
test react_loop::tests::test_react_loop_with_tool_call ... ok
test react_loop::tests::test_react_loop_with_output_finding ... ok
test react_loop::tests::test_react_loop_max_turns ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 六、设计决策文档

### 1. 为什么选择这种裁剪策略？

**问题**: 上下文窗口溢出时，哪些消息应该保留？

**方案**: 按优先级裁剪：System Prompt > 工具结果 > 最近对话 > 早期对话 > 思考文本

**对比替代方案**:
- **FIFO 裁剪**: 简单但会丢失重要的工具结果
- **按 token 大小裁剪**: 可能误删关键的短消息
- **智能语义裁剪**: 需要额外的 NLP 模型，增加复杂性

**选择理由**: 当前策略在"正确性"和"复杂度"之间取得平衡，确保 Agent 总能看到最重要的上下文。

### 2. 为什么流式解析要最后才反序列化？

**问题**: tool call 的 arguments 是增量 JSON，何时反序列化？

**方案**: 在所有帧接收完成后，对累积的完整字符串进行一次反序列化

**对比替代方案**:
- **每帧尝试解析**: 频繁失败，效率低
- **JSON 流式解析器**: 增加依赖复杂度
- **正则提取**: 不够通用，难以处理嵌套结构

**选择理由**: 简单可靠，`serde_json` 对完整 JSON 的解析效率很高，且符合大多数 LLM API 的 SSE 格式。

### 3. 为什么用三层停止条件？

**问题**: Agent 循环什么时候应该终止？

**方案**: 正常完成 + 循环上限 + 资源限制

**对比替代方案**:
- **单一条件**: 不够健壮，可能遗漏某些边界情况
- **更多条件**: 增加复杂性，维护成本高

**选择理由**: 三层条件覆盖了所有主要场景：正常工作流结束、异常循环防护、资源约束。

### 4. 局限性

- **token 估算精度**: 当前使用 `chars / 4` 的简单估算，实际 token 数可能有 ±10% 误差
- **裁剪粒度**: 按消息级别裁剪，无法精细到句子级别
- **流式解析容错**: 网络丢帧会导致 JSON 解析失败，需要额外的恢复机制
