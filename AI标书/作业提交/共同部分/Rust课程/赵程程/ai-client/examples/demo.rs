use ai_client::{ChatMessage, LlmClient, ToolDef};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== 场景1：普通对话 ===");
    demo_chat().await?;

    println!("\n=== 场景2：工具调用 ===");
    demo_tool_call().await?;

    Ok(())
}

async fn demo_chat() -> anyhow::Result<()> {
    let client = LlmClient::from_env()?;

    let messages = vec![
        ChatMessage::system("你是一个有帮助的AI助手。"),
        ChatMessage::user("用一句话介绍 Rust。"),
    ];

    let response = client.chat(&messages, &[]).await?;
    if let Some(content) = &response.content {
        println!("LLM: {}", content);
    }

    Ok(())
}

async fn demo_tool_call() -> anyhow::Result<()> {
    let client = LlmClient::from_env()?;

    let messages = vec![
        ChatMessage::system("你是一个有帮助的AI助手。"),
        ChatMessage::user("现在几点了？"),
    ];

    let tools = vec![ToolDef {
        name: "get_time".into(),
        description: "获取当前时间".into(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "timezone": {
                    "type": "string",
                    "description": "时区，如 Asia/Shanghai"
                }
            }
        }),
    }];

    let response = client.chat(&messages, &tools).await?;

    if !response.tool_calls.is_empty() {
        for tc in &response.tool_calls {
            println!("LLM 想调用工具: {} ({})", tc.name, tc.arguments);
        }
    } else if let Some(content) = &response.content {
        println!("LLM: {}", content);
    }

    Ok(())
}
