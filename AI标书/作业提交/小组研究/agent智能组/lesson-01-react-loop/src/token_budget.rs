use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct TokenBudget {
    max_tokens: usize,
    used: usize,
    reserved: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    System,
    User,
    Assistant,
    ToolResult,
    ToolCall,
}

#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub msg_type: MessageType,
    pub content: String,
    pub tokens: usize,
}

#[derive(Debug, Clone)]
pub struct TokenBudgetManager {
    budget: TokenBudget,
    messages: VecDeque<MessageRecord>,
    system_prompt: String,
    system_tokens: usize,
}

impl TokenBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            used: 0,
            reserved: 0,
        }
    }

    pub fn consume_system(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if tokens > self.max_tokens {
            return Err(BudgetError::SystemPromptTooLarge(tokens, self.max_tokens));
        }
        self.reserved = tokens;
        self.used += tokens;
        Ok(())
    }

    pub fn consume_user(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if self.used + tokens > self.max_tokens {
            return Err(BudgetError::InsufficientBudget);
        }
        self.used += tokens;
        Ok(())
    }

    pub fn consume_assistant(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if self.used + tokens > self.max_tokens {
            return Err(BudgetError::InsufficientBudget);
        }
        self.used += tokens;
        Ok(())
    }

    pub fn consume_tool_result(&mut self, text: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(text);
        if self.used + tokens > self.max_tokens {
            return Err(BudgetError::InsufficientBudget);
        }
        self.used += tokens;
        Ok(())
    }

    pub fn can_fit(&self, text: &str) -> bool {
        self.used + count_tokens(text) <= self.max_tokens
    }

    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.used)
    }

    pub fn used(&self) -> usize {
        self.used
    }

    pub fn reserved(&self) -> usize {
        self.reserved
    }
}

impl TokenBudgetManager {
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

    pub fn add_message(&mut self, msg_type: MessageType, content: &str) -> Result<(), BudgetError> {
        let tokens = count_tokens(content);
        
        if self.budget.remaining() < tokens + 100 {
            self.trim_messages()?;
        }

        if !self.budget.can_fit(content) {
            return Err(BudgetError::InsufficientBudget);
        }

        match msg_type {
            MessageType::System => self.budget.consume_system(content)?,
            MessageType::User => self.budget.consume_user(content)?,
            MessageType::Assistant => self.budget.consume_assistant(content)?,
            MessageType::ToolResult => self.budget.consume_tool_result(content)?,
            MessageType::ToolCall => self.budget.consume_assistant(content)?,
        }

        self.messages.push_back(MessageRecord {
            msg_type,
            content: content.to_string(),
            tokens,
        });

        Ok(())
    }

    fn trim_messages(&mut self) -> Result<(), BudgetError> {
        let target_remaining = self.budget.max_tokens / 2;

        // 计算需要保护的最近消息数量（最近 3 轮 = 6 条非 System 消息）
        let protect_recent = 6;
        let non_system_count = self.messages.iter()
            .filter(|m| m.msg_type != MessageType::System)
            .count();
        let protected_count = non_system_count.min(protect_recent);

        while self.budget.used() > target_remaining {
            let mut removed = false;

            // 从前往后找可删除的消息：跳过 System、ToolResult、以及最近 protected_count 条
            let mut non_system_seen = 0;
            let mut idx_to_remove = None;
            for (i, msg) in self.messages.iter().enumerate() {
                if msg.msg_type == MessageType::System {
                    continue;
                }
                if matches!(msg.msg_type, MessageType::ToolResult) {
                    continue;
                }
                non_system_seen += 1;
                // 跳过最近 protected_count 条非 System 消息
                if non_system_seen > non_system_count.saturating_sub(protected_count) {
                    continue;
                }
                idx_to_remove = Some(i);
                break;
            }

            if let Some(idx) = idx_to_remove {
                if let Some(removed_msg) = self.messages.remove(idx) {
                    self.budget.used -= removed_msg.tokens;
                    removed = true;
                }
            }

            if !removed {
                break;
            }
        }

        // 确保 System Prompt 仍在
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

    pub fn remaining(&self) -> usize {
        self.budget.remaining()
    }

    pub fn used(&self) -> usize {
        self.budget.used()
    }

    pub fn has_system_prompt(&self) -> bool {
        self.messages.iter().any(|m| m.msg_type == MessageType::System)
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn messages(&self) -> &VecDeque<MessageRecord> {
        &self.messages
    }
}

#[derive(Debug, PartialEq)]
pub enum BudgetError {
    InsufficientBudget,
    SystemPromptTooLarge(usize, usize),
}

fn count_tokens(text: &str) -> usize {
    text.chars().count() / 4 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_basic() {
        let mut budget = TokenBudget::new(100);
        assert_eq!(budget.remaining(), 100);
        
        budget.consume_system("Hello").unwrap();
        assert_eq!(budget.reserved(), 2);
        assert_eq!(budget.used(), 2);
        assert_eq!(budget.remaining(), 98);
    }

    #[test]
    fn test_budget_manager_add_messages() {
        let system_prompt = "You are a helpful assistant.";
        let mut manager = TokenBudgetManager::new(100, system_prompt);
        
        manager.add_message(MessageType::User, "Hello").unwrap();
        manager.add_message(MessageType::Assistant, "Hi there!").unwrap();
        
        assert_eq!(manager.message_count(), 3);
        assert!(manager.has_system_prompt());
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
    }

    #[test]
    fn test_budget_manager_trim_preserves_recent_turns() {
        let system_prompt = "You are a helpful assistant.";
        let mut manager = TokenBudgetManager::new(100, system_prompt);

        // 添加 20 轮对话，触发裁剪
        for i in 0..20 {
            let _ = manager.add_message(MessageType::User, &format!("User message number {}", i));
            let _ = manager.add_message(MessageType::Assistant, &format!("Assistant reply number {}", i));
        }

        // 验证最近 3 轮（最后 6 条非 System 消息）仍保留
        let non_system: Vec<_> = manager.messages()
            .iter()
            .filter(|m| m.msg_type != MessageType::System)
            .collect();

        // 至少保留 6 条最近消息
        assert!(non_system.len() >= 6, "应保留至少 6 条最近消息，实际 {}", non_system.len());

        // 验证最后几条是最近的消息（编号接近 19）
        let last_user = non_system.iter().rev()
            .find(|m| m.msg_type == MessageType::User)
            .unwrap();
        assert!(last_user.content.contains("19"), "最近的 User 消息应被保留");
    }
}
