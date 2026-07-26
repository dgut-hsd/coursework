use crate::token_budget::{TokenBudgetManager, MessageType};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Message {
    System(String),
    User(String),
    Assistant(String),
    Tool {
        id: String,
        name: String,
        arguments: Value,
    },
    ToolResult {
        tool_call_id: String,
        name: String,
        result: String,
    },
}

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub system_prompt: String,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone)]
pub struct ReActConfig {
    pub max_turns: usize,
    pub max_tokens: usize,
    pub min_tokens_for_next_turn: usize,
    pub tool_choice: ToolChoice,
}

#[derive(Debug, Clone)]
pub enum ToolChoice {
    Auto,
    Required,
    None,
}

impl Default for ReActConfig {
    fn default() -> Self {
        Self {
            max_turns: 15,
            max_tokens: 32768,
            min_tokens_for_next_turn: 1000,
            tool_choice: ToolChoice::Auto,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub trait LlmClient {
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        tool_choice: &ToolChoice,
    ) -> Result<ChatResponse, LlmError>;
}

#[derive(Debug)]
pub enum LlmError {
    NetworkError,
    InvalidResponse(String),
    RateLimited,
    Unknown(String),
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Fn(Value) -> Result<String, ToolError> + Send + Sync>>,
}

#[derive(Debug)]
pub enum ToolError {
    ToolNotFound(String),
    InvalidArguments(String),
    ExecutionError(String),
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(Value) -> Result<String, ToolError> + Send + Sync + 'static,
    {
        self.tools.insert(name.to_string(), Box::new(handler));
    }

    pub fn execute(&self, name: &str, arguments: Value) -> Result<String, ToolError> {
        let handler = self
            .tools
            .get(name)
            .ok_or(ToolError::ToolNotFound(name.to_string()))?;
        handler(arguments)
    }

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

pub async fn execute_react_loop(
    client: &impl LlmClient,
    agent_def: &AgentDefinition,
    user_query: &str,
    tools: &ToolRegistry,
    config: &ReActConfig,
) -> Result<Vec<Message>, ReactLoopError> {
    let mut messages = Vec::new();
    let mut budget_manager = TokenBudgetManager::new(config.max_tokens, &agent_def.system_prompt);

    messages.push(Message::System(agent_def.system_prompt.clone()));
    budget_manager
        .add_message(MessageType::System, &agent_def.system_prompt)
        .map_err(|_| ReactLoopError::BudgetError)?;

    messages.push(Message::User(user_query.to_string()));
    budget_manager
        .add_message(MessageType::User, user_query)
        .map_err(|_| ReactLoopError::BudgetError)?;

    for turn in 0..config.max_turns {
        let response = client
            .chat(&messages, &agent_def.tools, &config.tool_choice)
            .await
            .map_err(|e| ReactLoopError::LlmError(e))?;

        if let Some(tool_calls) = response.tool_calls {
            for tc in tool_calls {
                if tc.function.name == "output_finding" {
                    messages.push(Message::Tool {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    });
                    return Ok(messages);
                }

                let result = tools
                    .execute(&tc.function.name, tc.function.arguments.clone())
                    .map_err(|e| ReactLoopError::ToolError(e))?;

                messages.push(Message::Tool {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: tc.function.arguments.clone(),
                });
                budget_manager
                    .add_message(MessageType::ToolCall, &format!("{}({})", tc.function.name, tc.function.arguments))
                    .map_err(|_| ReactLoopError::BudgetError)?;

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
            if let Some(content) = response.content {
                messages.push(Message::Assistant(content.clone()));
                budget_manager
                    .add_message(MessageType::Assistant, &content)
                    .map_err(|_| ReactLoopError::BudgetError)?;
            }
        }

        if budget_manager.remaining() < config.min_tokens_for_next_turn {
            break;
        }
    }

    Ok(messages)
}

#[derive(Debug)]
pub enum ReactLoopError {
    LlmError(LlmError),
    ToolError(ToolError),
    BudgetError,
    MaxTurnsReached,
}

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
    }
}
