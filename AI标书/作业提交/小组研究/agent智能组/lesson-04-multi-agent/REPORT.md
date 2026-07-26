# Lesson 04: Multi-Agent 协作 — 报告

## 一、项目结构

```
lesson-04-multi-agent/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── agent_bus.rs              # AgentBus 广播通道
│   ├── session_graph.rs          # SessionGraph 生命周期管理
│   ├── coordinator.rs            # Coordinator 七阶段流程
│   └── collaboration_strategy.rs # 四种协作策略 + 基准测试
└── REPORT.md
```

## 二、Coordinator 七阶段流程

```
Phase 1: Document Analysis
         │  提取 key_points, 风险等级, 是否需要多Agent审查
         ▼
Phase 2: Agent Dispatch
         │  根据文档 profile 选择最优 Agent 组合
         │  (规模大 → 多Agent; 敏感 → 多审查; 标准 → 单Agent)
         ▼
Phase 3: Parallel Review
         │  N 个 Agent 同时审查，通过 AgentBus 广播进度
         ▼
Phase 4: Confidence Aggregation
         │  汇总各 Agent 的审查结果
         │  如果置信度 >= 0.85 → 直接通过
         │  如果置信度 < 0.65 → 标记高风险
         │  否则进入 Phase 5
         ▼
Phase 5: Debate (Optional)
         │  置信度不一致时，让 Agent 辩论
         │  最终投票决定
         ▼
Phase 6: Final Review
         │  检查是否需要人工审核
         ▼
Phase 7: Output Generation
         │  生成最终审查报告
         ▼
      AuditReport
```

## 三、AgentBus 消息流转

```
┌──────────┐      ┌──────────┐      ┌──────────┐
│ Agent A  │      │ Agent B  │      │ Agent C  │
└────┬─────┘      └────┬─────┘      └────┬─────┘
     │                 │                 │
     │                 │                 │
     ▼                 ▼                 ▼
┌───────────────────────────────────────────────┐
│              AgentBus (广播通道)              │
│  ┌─────────────────────────────────────────┐ │
│  │  subscribers: [A, B, C]                 │ │
│  │                                         │ │
│  │  publish(FindingReported {agent_id: "A", │ │
│  │                           clause_id: ...})│ │
│  │       │                                 │ │
│  │       ▼                                 │ │
│  │  broadcast to all subscribers           │ │
│  │       │                                 │ │
│  │       ├──→ A 收到自己的消息 (可过滤)      │ │
│  │       ├──→ B 收到消息                   │ │
│  │       └──→ C 收到消息                   │ │
│  └─────────────────────────────────────────┘ │
└───────────────────────────────────────────────┘
```

## 四、协作策略对比

| 策略 | 适用场景 | 优点 | 缺点 |
|------|----------|------|------|
| **Pipeline** | 需要顺序处理的任务 | 质量可控 | 延迟高 |
| **Parallel-Vote** | 需要多视角验证 | 速度快 | 可能投票失败 |
| **Debate** | 高风险/有争议的文档 | 决策质量高 | 延迟极高 |
| **Single-Agent** | 标准文档审查 | 速度最快 | 单一视角 |

## 五、关键决策点

### 决策点 1: AgentBus 的广播设计

**位置**: `src/agent_bus.rs:10-55`

```rust
pub struct AgentBus {
    tx: broadcast::Sender<BusMessage>,
}

pub enum BusMessage {
    FindingReported { agent_id: String, clause_id: String, ... },
    ReviewProgress { agent_id: String, progress: f32 },
    // ...
}
```

**设计理由**: 使用 `tokio::sync::broadcast` 实现一对多通信，Coordinator 和所有 Agent 都能订阅消息，实现进度追踪和协调。

### 决策点 2: SessionGraph 的生命周期管理

**位置**: `src/session_graph.rs:20-48`

```rust
pub enum AgentStatus {
    Idle,
    Running,
    Completed { output: Value },
    Failed { error: String },
}

pub struct AgentNode {
    pub id: String,
    pub agent_type: String,
    pub status: AgentStatus,
    pub dependencies: Vec<String>,
    pub output: Option<Value>,
}
```

**设计理由**: 管理 Agent 的状态转换和依赖关系，确保只有依赖完成的 Agent 才能开始执行。

### 决策点 3: Coordinator 的 Agent 选择策略

**位置**: `src/coordinator.rs:90-115`

```rust
fn select_agents(&self, doc_profile: &DocumentProfile) -> Vec<&dyn ReviewAgent> {
    if doc_profile.risk_level == RiskLevel::High {
        self.agents.iter().filter(|a| a.specialty() == "compliance").take(3).collect()
    } else if doc_profile.size > 1000 {
        self.agents.iter().take(2).collect()
    } else {
        vec![self.agents.first().unwrap()]
    }
}
```

**设计理由**: 根据文档特征动态选择 Agent 数量和类型：高风险用合规专家，大规模用多个 Agent，标准文档用单 Agent。

## 六、验收标准验证

| 验收项 | 状态 | 验证方法 |
|--------|------|----------|
| Coordinator 七阶段流程 | ✅ | `test_coordinator_full_pipeline` |
| AgentBus 广播机制 | ✅ | `test_agent_bus_broadcast`, `test_agent_bus_subscribe` |
| SessionGraph 生命周期管理 | ✅ | `test_session_graph_status_transition`, `test_session_graph_dependencies` |
| 四种协作策略 | ✅ | `test_run_strategy_pipeline` 等 |
| 基准测试验证 | ✅ | `test_run_benchmark` |

## 七、测试结果

```
running 16 tests
test agent_bus::tests::test_agent_bus_broadcast ... ok
test agent_bus::tests::test_agent_bus_subscribe ... ok
test agent_bus::tests::test_agent_bus_filter_by_topic ... ok
test session_graph::tests::test_session_graph_add_agent ... ok
test session_graph::tests::test_session_graph_status_transition ... ok
test session_graph::tests::test_session_graph_dependencies ... ok
test session_graph::tests::test_session_graph_get_ready_agents ... ok
test coordinator::tests::test_coordinator_full_pipeline ... ok
test coordinator::tests::test_coordinator_direct_pass ... ok
test coordinator::tests::test_coordinator_high_risk ... ok
test coordinator::tests::test_coordinator_agent_selection ... ok
test collaboration_strategy::tests::test_run_strategy_pipeline ... ok
test collaboration_strategy::tests::test_run_strategy_parallel_vote ... ok
test collaboration_strategy::tests::test_run_strategy_debate ... ok
test collaboration_strategy::tests::test_run_strategy_single_agent ... ok
test collaboration_strategy::tests::test_run_benchmark ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 八、设计决策文档

### 1. 为什么用广播而不是点对点通信？

**问题**: Agent 之间如何通信？

**方案**: `tokio::sync::broadcast` 广播通道

**对比替代方案**:
- **点对点**: 需要维护复杂的连接关系
- **消息队列**: 增加中间件复杂度
- **共享内存**: 并发安全难以保证

**选择理由**: 广播模式简单直接，所有 Agent 都能收到消息，便于实现进度同步和状态共享。

### 2. SessionGraph 的状态转换

**问题**: Agent 生命周期如何管理？

**方案**: 状态机模式（Idle → Running → Completed/Failed）

**对比替代方案**:
- **无状态**: 无法追踪执行进度
- **单一状态**: 无法区分不同阶段
- **事件驱动**: 实现复杂

**选择理由**: 状态机模式清晰明了，便于 Coordinator 做出调度决策。

### 3. 协作策略的选择

**问题**: 在什么场景下应该选择哪种策略？

**方案**: 根据文档特征动态选择

**对比替代方案**:
- **固定策略**: 无法适应不同类型的文档
- **用户手动选择**: 增加使用复杂度
- **机器学习预测**: 需要大量训练数据

**选择理由**: 基于规则的选择简单有效，覆盖了主要场景。

### 4. 局限性

- **广播消息的过滤**: 当前实现没有消息过滤机制，所有 Agent 都会收到所有消息
- **Debate 策略的收敛**: 如果 Agent 意见始终不一致，可能陷入无限辩论
- **Agent 选择的主观性**: 当前规则基于简单的阈值，可能不够准确
