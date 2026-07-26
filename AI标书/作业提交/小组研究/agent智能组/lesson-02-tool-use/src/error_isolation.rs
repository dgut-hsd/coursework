use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

use crate::tool_registry::{AgentTool, ToolError};

#[derive(Debug, Clone)]
pub enum ToolResult {
    Success(Value),
    Error(String),
    Timeout(String),
}

pub async fn execute_tool_safe(
    tool: &dyn AgentTool,
    args: &Value,
    timeout_duration: Duration,
) -> ToolResult {
    match timeout(timeout_duration, tool.execute(args.clone())).await {
        Ok(join_result) => match join_result {
            Ok(tool_result) => match tool_result {
                Ok(v) => ToolResult::Success(v),
                Err(e) => ToolResult::Error(format!("{}: {}", tool.name(), e)),
            },
            Err(_) => ToolResult::Error(format!("{}: task failed", tool.name())),
        },
        Err(_) => ToolResult::Timeout(tool.name().to_string()),
    }
}

pub async fn execute_tool_with_retry(
    tool: &dyn AgentTool,
    args: &Value,
    timeout_duration: Duration,
    max_retries: usize,
) -> ToolResult {
    for attempt in 0..=max_retries {
        let result = execute_tool_safe(tool, args, timeout_duration).await;
        
        match &result {
            ToolResult::Success(_) => return result,
            ToolResult::Error(_) if attempt < max_retries => {
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                continue;
            }
            ToolResult::Timeout(_) if attempt < max_retries => {
                tokio::time::sleep(Duration::from_millis(200 * (attempt + 1) as u64)).await;
                continue;
            }
            _ => return result,
        }
    }
    
    ToolResult::Error(format!("{}: max retries exceeded", tool.name()))
}

pub struct FlakySearchTool {
    fail_probability: f64,
    timeout_probability: f64,
}

impl FlakySearchTool {
    pub fn new(fail_probability: f64, timeout_probability: f64) -> Self {
        Self {
            fail_probability,
            timeout_probability,
        }
    }
}

impl AgentTool for FlakySearchTool {
    fn name(&self) -> &str {
        "flaky_search"
    }

    fn definition(&self) -> Value {
        serde_json::json!({
            "name": "flaky_search",
            "description": "A search tool that sometimes fails",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }
        })
    }

    fn execute(&self, args: Value) -> tokio::task::JoinHandle<Result<Value, ToolError>> {
        let fail_probability = self.fail_probability;
        let timeout_probability = self.timeout_probability;
        
        tokio::spawn(async move {
            let rand_val = rand::random::<f64>();
            
            if rand_val < timeout_probability {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            
            if rand_val < fail_probability {
                return Err(ToolError::ExecutionError("Service unavailable".to_string()));
            }

            let query = args.get("query")
                .and_then(|v| v.as_str())
                .ok_or(ToolError::InvalidArguments("query is required".to_string()))?;

            Ok(serde_json::json!({
                "query": query,
                "results": ["Mock result for flaky search"]
            }))
        })
    }
}

pub struct NoRetryTool;

impl AgentTool for NoRetryTool {
    fn name(&self) -> &str {
        "no_retry_tool"
    }

    fn definition(&self) -> Value {
        serde_json::json!({
            "name": "no_retry_tool",
            "description": "A tool that should not be retried",
            "parameters": {
                "type": "object",
                "properties": {
                    "value": {"type": "string"}
                }
            }
        })
    }

    fn execute(&self, args: Value) -> tokio::task::JoinHandle<Result<Value, ToolError>> {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            let value = args.get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            Ok(serde_json::json!({
                "result": format!("processed: {}", value)
            }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_tool_safe_success() {
        let tool = crate::tool_registry::SearchKnowledgeTool;
        let args = serde_json::json!({"query": "test"});
        
        let result = execute_tool_safe(&tool, &args, Duration::from_secs(5)).await;
        assert!(matches!(result, ToolResult::Success(_)));
    }

    #[tokio::test]
    async fn test_execute_tool_safe_timeout() {
        let tool = FlakySearchTool::new(0.0, 1.0);
        let args = serde_json::json!({"query": "test"});
        
        let result = execute_tool_safe(&tool, &args, Duration::from_millis(100)).await;
        assert!(matches!(result, ToolResult::Timeout(_)));
    }

    #[tokio::test]
    async fn test_execute_tool_safe_error() {
        let tool = FlakySearchTool::new(1.0, 0.0);
        let args = serde_json::json!({"query": "test"});
        
        let result = execute_tool_safe(&tool, &args, Duration::from_secs(5)).await;
        assert!(matches!(result, ToolResult::Error(_)));
    }

    #[tokio::test]
    async fn test_execute_tool_with_retry_success_after_retry() {
        let tool = FlakySearchTool::new(0.5, 0.0);
        let args = serde_json::json!({"query": "test"});
        
        let result = execute_tool_with_retry(&tool, &args, Duration::from_secs(1), 3).await;
        assert!(matches!(result, ToolResult::Success(_)));
    }

    #[tokio::test]
    async fn test_error_isolation_does_not_crash() {
        let tool = FlakySearchTool::new(1.0, 0.0);
        let args = serde_json::json!({"query": "test"});
        
        let result = execute_tool_safe(&tool, &args, Duration::from_secs(5)).await;
        
        match result {
            ToolResult::Error(msg) => {
                assert!(msg.contains("flaky_search"));
            }
            _ => panic!("Expected error"),
        }
    }
}
