use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// LLM 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum ChatMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        tool_call_id: String,
        content: String,
    },
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        ChatMessage::System {
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        ChatMessage::User {
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        ChatMessage::Assistant {
            content: Some(content.to_string()),
            tool_calls: None,
        }
    }

    pub fn tool(tool_call_id: &str, content: &str) -> Self {
        ChatMessage::Tool {
            tool_call_id: tool_call_id.to_string(),
            content: content.to_string(),
        }
    }
}

/// 工具定义（发送给 LLM）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl ToolDef {
    pub fn to_request_format(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters
            }
        })
    }
}

/// LLM 返回的工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// LLM 响应
#[derive(Debug)]
pub struct LlmResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

/// DashScope API 客户端
pub struct LlmClient {
    api_key: String,
    model: String,
    client: Client,
}

impl LlmClient {
    /// 从环境变量创建客户端
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let api_key = std::env::var("DASHSCOPE_API_KEY")
            .context("未找到 DASHSCOPE_API_KEY 环境变量，请在 .env 文件中设置")?;

        let model = std::env::var("DASHSCOPE_MODEL")
            .unwrap_or_else(|_| "qwen-plus".to_string());

        Ok(Self {
            api_key,
            model,
            client: Client::new(),
        })
    }

    /// 调用 DashScope API
    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDef],
    ) -> Result<LlmResponse> {
        let endpoint = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";

        let mut body = serde_json::json!({
            "model": self.model,
            "input": {
                "messages": messages
            },
            "parameters": {
                "result_format": "message"
            }
        });

        if !tools.is_empty() {
            let tools_json: Vec<serde_json::Value> =
                tools.iter().map(|t| t.to_request_format()).collect();
            body["input"]["tools"] = serde_json::json!(tools_json);
        }

        let resp = self
            .client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("发送 HTTP 请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("API 返回 {}: {}", status, text);
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .context("解析 API 响应失败")?;

        let output = &result["output"];
        let choices = &output["choices"];

        let choice = choices
            .get(0)
            .context("API 返回的 choices 为空")?;

        let message = &choice["message"];

        let content = message["content"]
            .as_str()
            .map(|s| s.to_string());

        let mut tool_calls = Vec::new();
        if let Some(tc_array) = message["tool_calls"].as_array() {
            for tc in tc_array {
                tool_calls.push(ToolCall {
                    id: tc["id"].as_str().unwrap_or("").to_string(),
                    name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: tc["function"]["arguments"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }

        Ok(LlmResponse {
            content,
            tool_calls,
        })
    }
}
