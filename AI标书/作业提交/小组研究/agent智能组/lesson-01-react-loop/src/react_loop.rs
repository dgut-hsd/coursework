use crate::token_budget::{TokenBudgetManager, MessageType};
use serde_json::Value;
use std::collections::HashMap;

/// 消息类型枚举 —— 表示对话中的各种消息
#[derive(Debug, Clone)]
pub enum Message {
    /// 系统消息 —— 告诉 AI 它是谁、要做什么（身份设定）
    System(String),
    /// 用户消息 —— 用户提出的问题或指令
    User(String),
    /// 助手消息 —— AI 的思考过程和回复
    Assistant(String),
    /// 工具调用消息 —— AI 请求调用外部工具
    Tool {
        /// 工具调用的唯一标识
        id: String,
        /// 工具名称（如 "search_knowledge"）
        name: String,
        /// 工具参数（JSON 格式）
        arguments: Value,
    },
    /// 工具结果消息 —— 工具执行完后返回的结果
    ToolResult {
        /// 对应哪个工具调用的结果
        tool_call_id: String,
        /// 工具名称
        name: String,
        /// 工具返回的结果内容
        result: String,
    },
}

/// Agent 定义 —— 描述一个 AI 助手的配置
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    /// 系统提示词 —— AI 的身份描述和行为准则
    pub system_prompt: String,
    /// 可用工具列表 —— AI 可以调用的工具
    pub tools: Vec<ToolDefinition>,
}

/// 工具定义 —— 描述一个工具的元数据
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    /// 工具名称（用于调用）
    pub name: String,
    /// 工具描述（告诉 AI 这个工具是做什么的）
    pub description: String,
    /// 工具参数定义（JSON Schema 格式）
    pub parameters: Value,
}

/// ReAct 循环配置 —— 控制循环行为的参数
#[derive(Debug, Clone)]
pub struct ReActConfig {
    /// 最大轮数 —— 防止无限循环
    pub max_turns: usize,
    /// 最大 Token 数 —— 上下文窗口大小（如 32K）
    pub max_tokens: usize,
    /// 下一轮所需的最小 Token —— 低于这个值就停止循环
    pub min_tokens_for_next_turn: usize,
    /// 工具选择策略 —— AI 是否必须调用工具
    pub tool_choice: ToolChoice,
}

/// 工具选择策略枚举
#[derive(Debug, Clone)]
pub enum ToolChoice {
    /// 自动 —— AI 自己决定是否调用工具
    Auto,
    /// 必须调用 —— AI 必须调用一个工具
    Required,
    /// 不调用工具 —— AI 只能直接回答
    None,
}

impl Default for ReActConfig {
    fn default() -> Self {
        Self {
            max_turns: 15,              // 默认最多 15 轮
            max_tokens: 32768,          // 默认 32K Token
            min_tokens_for_next_turn: 1000, // 剩余不足 1000 Token 就停止
            tool_choice: ToolChoice::Auto,  // 默认自动选择
        }
    }
}

/// 工具调用结构 —— AI 请求调用工具时的完整信息
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// 调用 ID（用于关联结果）
    pub id: String,
    /// 函数调用信息
    pub function: FunctionCall,
}

/// 函数调用结构 —— 具体要调用哪个函数以及参数
#[derive(Debug, Clone)]
pub struct FunctionCall {
    /// 函数名称（工具名称）
    pub name: String,
    /// 函数参数
    pub arguments: Value,
}

/// 聊天响应 —— 大模型返回的结果
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// 文本内容（AI 的回复）
    pub content: Option<String>,
    /// 工具调用列表（AI 请求调用的工具）
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// LLM 客户端接口 —— 定义与大模型通信的方法
pub trait LlmClient {
    /// 发送聊天请求给大模型
    /// 参数：
    /// - messages: 对话历史消息
    /// - tools: 可用工具列表
    /// - tool_choice: 工具选择策略
    /// 返回：大模型的响应
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        tool_choice: &ToolChoice,
    ) -> Result<ChatResponse, LlmError>;
}

/// LLM 错误类型 —— 与大模型通信时可能出现的错误
#[derive(Debug)]
pub enum LlmError {
    /// 网络错误（如断网）
    NetworkError,
    /// 无效响应（模型返回格式错误）
    InvalidResponse(String),
    /// 限流（请求太频繁被拒绝）
    RateLimited,
    /// 未知错误
    Unknown(String),
}

/// 工具注册表 —— 管理所有可用的工具
pub struct ToolRegistry {
    /// 工具映射表：工具名称 → 工具处理函数
    tools: HashMap<String, Box<dyn Fn(Value) -> Result<String, ToolError> + Send + Sync>>,
}

/// 工具错误类型 —— 工具执行时可能出现的错误
#[derive(Debug)]
pub enum ToolError {
    /// 工具未找到（请求调用的工具不存在）
    ToolNotFound(String),
    /// 参数无效（工具参数格式错误）
    InvalidArguments(String),
    /// 执行错误（工具执行过程中出错）
    ExecutionError(String),
}

impl ToolRegistry {
    /// 创建一个新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册一个工具
    /// 参数：
    /// - name: 工具名称
    /// - handler: 工具处理函数（接收参数，返回结果）
    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(Value) -> Result<String, ToolError> + Send + Sync + 'static,
    {
        self.tools.insert(name.to_string(), Box::new(handler));
    }

    /// 执行一个工具
    /// 参数：
    /// - name: 工具名称
    /// - arguments: 工具参数
    /// 返回：工具执行结果
    pub fn execute(&self, name: &str, arguments: Value) -> Result<String, ToolError> {
        let handler = self
            .tools
            .get(name)
            .ok_or(ToolError::ToolNotFound(name.to_string()))?;
        handler(arguments)
    }

    /// 获取所有工具的定义列表（用于发送给大模型）
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .keys()
            .map(|name| ToolDefinition {
                name: name.clone(),
                description: "".to_string(),
                parameters: Value::Object(serde_json::Map::new()),
            })
            .collect()
    }
}

/// 执行 ReAct 主循环
/// 参数：
/// - client: LLM 客户端（与大模型通信）
/// - agent_def: Agent 定义（身份和工具配置）
/// - user_query: 用户问题
/// - tools: 工具注册表
/// - config: ReAct 循环配置
/// 返回：完整的对话消息历史
pub async fn execute_react_loop(
    client: &impl LlmClient,
    agent_def: &AgentDefinition,
    user_query: &str,
    tools: &ToolRegistry,
    config: &ReActConfig,
) -> Result<Vec<Message>, ReactLoopError> {
    // 初始化消息列表和 Token 预算管理器
    let mut messages = Vec::new();
    let mut budget_manager = TokenBudgetManager::new(config.max_tokens, &agent_def.system_prompt);

    // 添加系统消息（AI 的身份设定）
    messages.push(Message::System(agent_def.system_prompt.clone()));
    budget_manager
        .add_message(MessageType::System, &agent_def.system_prompt)
        .map_err(|_| ReactLoopError::BudgetError)?;

    // 添加用户消息（用户的问题）
    messages.push(Message::User(user_query.to_string()));
    budget_manager
        .add_message(MessageType::User, user_query)
        .map_err(|_| ReactLoopError::BudgetError)?;

    // 开始循环：思考 → 行动 → 观察
    for _turn in 0..config.max_turns {
        // 调用大模型获取响应
        let response = client
            .chat(&messages, &agent_def.tools, &config.tool_choice)
            .await
            .map_err(|e| ReactLoopError::LlmError(e))?;

        // 如果响应中包含工具调用
        if let Some(tool_calls) = response.tool_calls {
            for tc in tool_calls {
                // 特殊处理：如果调用的是 output_finding（输出最终结果），则结束循环
                if tc.function.name == "output_finding" {
                    messages.push(Message::Tool {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    });
                    return Ok(messages);
                }

                // 执行工具调用
                let result = tools
                    .execute(&tc.function.name, tc.function.arguments.clone())
                    .map_err(|e| ReactLoopError::ToolError(e))?;

                // 记录工具调用消息
                messages.push(Message::Tool {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: tc.function.arguments.clone(),
                });
                budget_manager
                    .add_message(MessageType::ToolCall, &format!("{}({})", tc.function.name, tc.function.arguments))
                    .map_err(|_| ReactLoopError::BudgetError)?;

                // 记录工具结果消息
                messages.push(Message::ToolResult {
                    tool_call_id: tc.id,
                    name: tc.function.name,
                    result: result.clone(),
                });
                budget_manager
                    .add_message(MessageType::ToolResult, &result)
                    .map_err(|_| ReactLoopError::BudgetError)?;
            }
        } else {
            // 如果响应是纯文本（没有工具调用）
            if let Some(content) = response.content {
                messages.push(Message::Assistant(content.clone()));
                budget_manager
                    .add_message(MessageType::Assistant, &content)
                    .map_err(|_| ReactLoopError::BudgetError)?;
            }
        }

        // 检查 Token 预算是否足够进行下一轮
        if budget_manager.remaining() < config.min_tokens_for_next_turn {
            break;
        }
    }

    Ok(messages)
}

/// ReAct 循环错误类型
#[derive(Debug)]
pub enum ReactLoopError {
    /// LLM 相关错误
    LlmError(LlmError),
    /// 工具相关错误
    ToolError(ToolError),
    /// Token 预算错误
    BudgetError,
    /// 达到最大轮数
    MaxTurnsReached,
}

/// 模拟 LLM 客户端（用于测试）
struct MockLlmClient {
    responses: Vec<ChatResponse>,
    index: std::sync::atomic::AtomicUsize,
}

impl MockLlmClient {
    fn new(responses: Vec<ChatResponse>) -> Self {
        Self { responses, index: std::sync::atomic::AtomicUsize::new(0) }
    }
}

impl LlmClient for MockLlmClient {
    async fn chat(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _tool_choice: &ToolChoice,
    ) -> Result<ChatResponse, LlmError> {
        let idx = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if idx >= self.responses.len() {
            return Ok(ChatResponse {
                content: Some("I have completed my analysis.".to_string()),
                tool_calls: None,
            });
        }
        Ok(self.responses[idx].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用 LLM 客户端（记录调用次数）
    struct TestLlmClient {
        responses: Vec<ChatResponse>,
        call_count: std::sync::atomic::AtomicUsize,
    }

    impl TestLlmClient {
        fn new(responses: Vec<ChatResponse>) -> Self {
            Self {
                responses,
                call_count: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }

    impl LlmClient for TestLlmClient {
        async fn chat(
            &self,
            _messages: &[Message],
            _tools: &[ToolDefinition],
            _tool_choice: &ToolChoice,
        ) -> Result<ChatResponse, LlmError> {
            let count = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count >= self.responses.len() {
                return Ok(ChatResponse {
                    content: Some("Done".to_string()),
                    tool_calls: None,
                });
            }
            Ok(self.responses[count].clone())
        }
    }

    #[tokio::test]
    async fn test_react_loop_with_tool_call() {
        println!("===测试 ReAct循环 ===");
        let agent_def = AgentDefinition {
            system_prompt: "You are a helpful assistant.".to_string(),
            tools: vec![ToolDefinition {
                name: "search".to_string(),
                description: "Search for information".to_string(),
                parameters: Value::Object(serde_json::Map::new()),
            }],
        };

        let mut tools = ToolRegistry::new();
        tools.register("search", |args| {
            Ok(format!("Search results for: {:?}", args))
        });

        let responses = vec![
            ChatResponse {
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    function: FunctionCall {
                        name: "search".to_string(),
                        arguments: serde_json::json!({"query": "test"}),
                    },
                }]),
            },
            ChatResponse {
                content: Some("Here are the results.".to_string()),
                tool_calls: None,
            },
        ];

        let client = TestLlmClient::new(responses);
        let config = ReActConfig::default();

        let messages = execute_react_loop(&client, &agent_def, "Search for test", &tools, &config)
            .await
            .unwrap();

        assert!(messages.len() >= 4);
        println!("✓ ReAct循环处理工具调用");
    }

    #[tokio::test]
    async fn test_react_loop_with_output_finding() {
        let agent_def = AgentDefinition {
            system_prompt: "You are a helpful assistant.".to_string(),
            tools: vec![ToolDefinition {
                name: "output_finding".to_string(),
                description: "Output final finding".to_string(),
                parameters: Value::Object(serde_json::Map::new()),
            }],
        };

        let mut tools = ToolRegistry::new();
        tools.register("output_finding", |args| {
            Ok(format!("Finding: {:?}", args))
        });

        let responses = vec![
            ChatResponse {
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    function: FunctionCall {
                        name: "output_finding".to_string(),
                        arguments: serde_json::json!({"summary": "Done"}),
                    },
                }]),
            },
        ];

        let client = TestLlmClient::new(responses);
        let config = ReActConfig::default();

        let messages = execute_react_loop(&client, &agent_def, "Analyze this", &tools, &config)
            .await
            .unwrap();

        let last_msg = messages.last().unwrap();
        if let Message::Tool { name, .. } = last_msg {
            assert_eq!(name, "output_finding");
        }
        println!("✓ ReAct循环通过输出工具结束");
    }

    #[tokio::test]
    async fn test_react_loop_max_turns() {
        let agent_def = AgentDefinition {
            system_prompt: "You are a helpful assistant.".to_string(),
            tools: vec![],
        };

        let tools = ToolRegistry::new();

        let responses = vec![
            ChatResponse {
                content: Some("Thinking...".to_string()),
                tool_calls: None,
            };
            20
        ];

        let client = TestLlmClient::new(responses);
        let config = ReActConfig {
            max_turns: 5,
            ..ReActConfig::default()
        };

        let messages = execute_react_loop(&client, &agent_def, "Test", &tools, &config)
            .await
            .unwrap();

        assert!(messages.len() <= 7);
        println!("✓ ReAct循环达到最大轮数停止");
    }

    #[tokio::test]
    async fn test_react_loop_budget_check() {
        let agent_def = AgentDefinition {
            system_prompt: "You are a helpful assistant.".to_string(),
            tools: vec![],
        };

        let tools = ToolRegistry::new();

        let responses = vec![
            ChatResponse {
                content: Some("A".repeat(1000)),
                tool_calls: None,
            };
            50
        ];

        let client = TestLlmClient::new(responses);
        let config = ReActConfig {
            max_tokens: 2000,
            min_tokens_for_next_turn: 100,
            ..ReActConfig::default()
        };

        let messages = execute_react_loop(&client, &agent_def, "Test", &tools, &config)
            .await
            .unwrap();

        assert!(messages.len() > 0);
        println!("✓ ReAct循环Token预算不足时停止");
    }
}
