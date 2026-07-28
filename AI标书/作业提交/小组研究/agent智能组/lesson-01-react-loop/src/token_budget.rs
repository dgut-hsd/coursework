use std::collections::VecDeque;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;

/// Token 预算结构 —— 跟踪 Token 使用情况
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// 最大 Token 数（上下文窗口大小）
    max_tokens: usize,
    /// 已使用的 Token 数
    used: usize,
    /// 保留的 Token 数（用于 System Prompt）
    reserved: usize,
}

/// 消息类型枚举 —— 区分不同类型的消息
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// 系统消息（身份设定）
    System,
    /// 用户消息（问题或指令）
    User,
    /// 助手消息（AI 的思考和回复）
    Assistant,
    /// 工具结果消息（工具执行返回的结果）
    ToolResult,
    /// 工具调用消息（AI 请求调用工具）
    ToolCall,
}

/// 消息记录 —— 存储单条消息的信息
#[derive(Debug, Clone)]
pub struct MessageRecord {
    /// 消息类型
    pub msg_type: MessageType,
    /// 消息内容
    pub content: String,
    /// 该消息占用的 Token 数
    pub tokens: usize,
}

/// Token 预算管理器 —— 管理消息历史和 Token 使用
#[derive(Debug, Clone)]
pub struct TokenBudgetManager {
    /// Token 预算
    budget: TokenBudget,
    /// 消息队列（按时间顺序存储）
    messages: VecDeque<MessageRecord>,
    /// System Prompt 内容（单独保存，确保裁剪后可以恢复）
    system_prompt: String,
    /// System Prompt 占用的 Token 数
    system_tokens: usize,
}

impl TokenBudget {
    /// 创建一个新的 Token 预算
    /// 参数：max_tokens — 最大 Token 数
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            used: 0,
            reserved: 0,
        }
    }

    /// 消耗 System 消息的 Token
    pub fn consume_system(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if tokens > self.max_tokens {
            return Err(BudgetError::SystemPromptTooLarge(tokens, self.max_tokens));
        }
        self.reserved = tokens;
        self.used += tokens;
        Ok(())
    }

    /// 消耗 User 消息的 Token
    pub fn consume_user(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if self.used + tokens > self.max_tokens {
            return Err(BudgetError::InsufficientBudget);
        }
        self.used += tokens;
        Ok(())
    }

    /// 消耗 Assistant 消息的 Token
    pub fn consume_assistant(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if self.used + tokens > self.max_tokens {
            return Err(BudgetError::InsufficientBudget);
        }
        self.used += tokens;
        Ok(())
    }

    /// 消耗 ToolResult 消息的 Token
    pub fn consume_tool_result(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if self.used + tokens > self.max_tokens {
            return Err(BudgetError::InsufficientBudget);
        }
        self.used += tokens;
        Ok(())
    }

    /// 检查文本是否可以放入剩余预算
    pub fn can_fit(&self, text: &str) -> bool {
        self.used + count_tokens(text) <= self.max_tokens
    }

    /// 获取剩余 Token 数
    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.used)
    }

    /// 获取已使用的 Token 数
    pub fn used(&self) -> usize {
        self.used
    }

    /// 获取保留的 Token 数
    pub fn reserved(&self) -> usize {
        self.reserved
    }
}

impl TokenBudgetManager {
    /// 创建一个新的 Token 预算管理器
    /// 参数：
    /// - max_tokens: 最大 Token 数
    /// - system_prompt: System Prompt 内容（会预先消耗 Token）
    pub fn new(max_tokens: usize, system_prompt: &str) -> Self {
        let system_tokens = count_tokens(system_prompt);
        let mut budget = TokenBudget::new(max_tokens);
        let _ = budget.consume_system(system_prompt);

        let mut messages = VecDeque::new();
        messages.push_back(MessageRecord {
            msg_type: MessageType::System,
            content: system_prompt.to_string(),
            tokens: system_tokens,
        });

        Self {
            budget,
            messages,
            system_prompt: system_prompt.to_string(),
            system_tokens,
        }
    }

    /// 添加一条消息到消息历史
    /// 参数：
    /// - msg_type: 消息类型
    /// - content: 消息内容
    /// 返回：成功或预算不足错误
    pub fn add_message(&mut self, msg_type: MessageType, content: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(content);
        
        // 如果剩余预算不足，先裁剪旧消息
        if self.budget.remaining() < tokens + 100 {
            self.trim_messages()?;
        }

        // 检查是否能放入
        if !self.budget.can_fit(content) {
            return Err(BudgetError::InsufficientBudget);
        }

        // 根据消息类型消耗 Token
        match msg_type {
            MessageType::System => self.budget.consume_system(content)?,
            MessageType::User => self.budget.consume_user(content)?,
            MessageType::Assistant => self.budget.consume_assistant(content)?,
            MessageType::ToolResult => self.budget.consume_tool_result(content)?,
            MessageType::ToolCall => self.budget.consume_assistant(content)?,
        }

        // 添加到消息队列
        self.messages.push_back(MessageRecord {
            msg_type,
            content: content.to_string(),
            tokens,
        });

        Ok(())
    }

    /// 裁剪消息历史 —— 当 Token 预算不足时删除旧消息
    /// 裁剪策略：
    /// 1. 必须保留：System Prompt（AI 的身份）
    /// 2. 优先保留：ToolResult（工具执行结果）
    /// 3. 优先保留：最近 3 轮对话（6 条消息）
    /// 4. 可以丢弃：早期的 User/Assistant 消息
    fn trim_messages(&mut self) -> Result<(), BudgetError> {
        // 目标：裁剪到剩余一半 Token
        let target_remaining = self.budget.max_tokens / 2;

        // 计算需要保护的最近消息数量（最近 3 轮 = 6 条非 System 消息）
        let protect_recent = 6;
        let non_system_count = self.messages.iter()
            .filter(|m| m.msg_type != MessageType::System)
            .count();
        let protected_count = non_system_count.min(protect_recent);

        // 循环删除，直到 Token 使用量降到目标以下
        while self.budget.used() > target_remaining {
            let mut removed = false;

            // 从前往后找可删除的消息：跳过 System、ToolResult、以及最近 protected_count 条
            let mut non_system_seen = 0;
            let mut idx_to_remove = None;
            for (i, msg) in self.messages.iter().enumerate() {
                // 跳过 System 消息
                if msg.msg_type == MessageType::System {
                    continue;
                }
                // 跳过 ToolResult 消息
                if matches!(msg.msg_type, MessageType::ToolResult) {
                    continue;
                }
                non_system_seen += 1;
                // 跳过最近 protected_count 条非 System 消息
                if non_system_seen > non_system_count.saturating_sub(protected_count) {
                    continue;
                }
                // 找到可以删除的消息
                idx_to_remove = Some(i);
                break;
            }

            // 删除消息并更新 Token 计数
            if let Some(idx) = idx_to_remove {
                if let Some(removed_msg) = self.messages.remove(idx) {
                    self.budget.used -= removed_msg.tokens;
                    removed = true;
                }
            }

            // 如果没有删除任何消息，停止循环
            if !removed {
                break;
            }
        }

        // 确保 System Prompt 仍在队列中（如果被意外删除，重新插入到队首）
        if !self.messages.iter().any(|m| m.msg_type == MessageType::System) {
            self.messages.push_front(MessageRecord {
                msg_type: MessageType::System,
                content: self.system_prompt.clone(),
                tokens: self.system_tokens,
            });
            self.budget.used += self.system_tokens;
        }

        Ok(())
    }

    /// 获取剩余 Token 数
    pub fn remaining(&self) -> usize {
        self.budget.remaining()
    }

    /// 获取已使用的 Token 数
    pub fn used(&self) -> usize {
        self.budget.used()
    }

    /// 检查是否包含 System Prompt
    pub fn has_system_prompt(&self) -> bool {
        self.messages.iter().any(|m| m.msg_type == MessageType::System)
    }

    /// 获取消息总数
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// 获取消息队列的不可变引用
    pub fn messages(&self) -> &VecDeque<MessageRecord> {
        &self.messages
    }
}

/// 预算错误类型
#[derive(Debug, PartialEq)]
pub enum BudgetError {
    /// Token 预算不足
    InsufficientBudget,
    /// System Prompt 太大（超过最大 Token 数）
    SystemPromptTooLarge(usize, usize),
}

/// 计算文本的 Token 数（简化版估算）
/// 真实场景应该使用专业的 Tokenizer，这里用简单估算：4 字符 ≈ 1 Token
fn count_tokens(text: &str) -> usize {
    text.chars().count() / 4 + 1
}

/// DashScope API 请求结构体
#[derive(Debug, Serialize)]
struct DashScopeRequest {
    model: String,
    input: DashScopeInput,
    parameters: DashScopeParameters,
}

#[derive(Debug, Serialize)]
struct DashScopeInput {
    messages: Vec<DashScopeMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DashScopeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct DashScopeParameters {
    return_token_usage: bool,
}

/// DashScope API 响应结构体
#[derive(Debug, Deserialize)]
struct DashScopeResponse {
    output: DashScopeOutput,
    usage: DashScopeUsage,
}

#[derive(Debug, Deserialize)]
struct DashScopeOutput {
    finish_reason: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct DashScopeUsage {
    input_tokens: usize,
    output_tokens: usize,
    total_tokens: usize,
}

/// 使用 DashScope API 验证 Token 计数准确性
/// 参数：text - 要验证的文本
/// 返回：(实际 Token 数, API 返回的 Token 数, 误差百分比)
pub async fn validate_token_count(text: &str) -> Result<(usize, usize, f64), DashScopeError> {
    dotenv().ok();
    
    let api_key = env::var("DASHSCOPE_API_KEY").ok();
    if api_key.is_none() {
        return Err(DashScopeError::MissingApiKey);
    }
    let api_key = api_key.unwrap();
    
    let client = Client::new();
    
    let request = DashScopeRequest {
        model: "qwen-turbo".to_string(),
        input: DashScopeInput {
            messages: vec![
                DashScopeMessage {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
        },
        parameters: DashScopeParameters {
            return_token_usage: true,
        },
    };
    
    let response = client
        .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| DashScopeError::RequestFailed(e.to_string()))?;
    
    let response_text = response.text().await.map_err(|e| DashScopeError::ResponseParseFailed(e.to_string()))?;
    
    let result: DashScopeResponse = serde_json::from_str(&response_text)
        .map_err(|e| DashScopeError::JsonParseFailed(format!("{}", e)))?;
    
    let api_tokens = result.usage.input_tokens;
    let local_tokens = count_tokens(text);
    let error_percent = ((local_tokens as f64 - api_tokens as f64).abs() / api_tokens as f64) * 100.0;
    
    Ok((local_tokens, api_tokens, error_percent))
}

/// DashScope API 错误类型
#[derive(Debug)]
pub enum DashScopeError {
    MissingApiKey,
    RequestFailed(String),
    ResponseParseFailed(String),
    JsonParseFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_basic() {
        println!("===测试 Token预算管理 ===");
        let mut budget = TokenBudget::new(100);
        assert_eq!(budget.remaining(), 100);
        
        budget.consume_system("Hello").unwrap();
        assert_eq!(budget.reserved(), 2);
        assert_eq!(budget.used(), 2);
        assert_eq!(budget.remaining(), 98);
        println!("✓ Token预算基本操作正常");
    }

    #[test]
    fn test_budget_manager_add_messages() {
        let system_prompt = "You are a helpful assistant.";
        let mut manager = TokenBudgetManager::new(100, system_prompt);
        
        manager.add_message(MessageType::User, "Hello").unwrap();
        manager.add_message(MessageType::Assistant, "Hi there!").unwrap();
        
        assert_eq!(manager.message_count(), 3);
        assert!(manager.has_system_prompt());
        println!("✓ 预算管理器可以添加消息");
    }

    #[test]
    fn test_budget_manager_trim_preserves_system() {
        let system_prompt = "You are a helpful assistant.";
        let mut manager = TokenBudgetManager::new(100, system_prompt);
        
        for i in 0..50 {
            let user_msg = format!("User message {}", i);
            let assistant_msg = format!("Assistant response {}", i);
            let _ = manager.add_message(MessageType::User, &user_msg);
            let _ = manager.add_message(MessageType::Assistant, &assistant_msg);
        }
        
        assert!(manager.has_system_prompt());
        assert!(manager.used() <= 100);
        println!("✓ 裁剪后 System Prompt 仍被保留");
    }

    #[test]
    fn test_budget_manager_trim_preserves_tool_results() {
        let system_prompt = "You are a helpful assistant.";
        let mut manager = TokenBudgetManager::new(100, system_prompt);
        
        for i in 0..20 {
            let _ = manager.add_message(MessageType::User, &format!("User {}", i));
            let _ = manager.add_message(MessageType::Assistant, &format!("Thinking {}", i));
            let _ = manager.add_message(MessageType::ToolResult, &format!("Result {}", i));
        }
        
        assert!(manager.has_system_prompt());
        
        let tool_results_count = manager.messages()
            .iter()
            .filter(|m| m.msg_type == MessageType::ToolResult)
            .count();
        assert!(tool_results_count > 0);
        println!("✓ 裁剪后 ToolResult 仍被保留");
    }

    #[test]
    fn test_budget_manager_trim_preserves_recent_turns() {
        let system_prompt = "You are a helpful assistant.";
        let mut manager = TokenBudgetManager::new(100, system_prompt);

        for i in 0..20 {
            let _ = manager.add_message(MessageType::User, &format!("User message number {}", i));
            let _ = manager.add_message(MessageType::Assistant, &format!("Assistant reply number {}", i));
        }

        let non_system: Vec<_> = manager.messages()
            .iter()
            .filter(|m| m.msg_type != MessageType::System)
            .collect();

        assert!(non_system.len() >= 6, "应保留至少 6 条最近消息，实际 {}", non_system.len());

        let last_user = non_system.iter().rev()
            .find(|m| m.msg_type == MessageType::User)
            .unwrap();
        assert!(last_user.content.contains("19"), "最近的 User 消息应被保留");
        println!("✓ 裁剪后最近 3 轮对话仍被保留");
    }
}
