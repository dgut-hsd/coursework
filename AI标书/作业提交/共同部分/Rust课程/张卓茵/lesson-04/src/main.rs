use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

// 1. 配置常量
// 阿里云 DashScope 的通义千问 API 地址
const ENDPOINT: &str = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";

// 2. 命令行参数定义
#[derive(Parser)]
#[command(name = "ai-assistant", about = "一个简单的命令行 AI 助手")]
struct Cli {
    /// 模型名称 (例如: qwen-plus, qwen-turbo)
    #[arg(long, default_value = "qwen-plus")]
    model: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 问答模式：向 AI 提问
    Ask {
        question: String,
    },
    /// 翻译模式：将文本翻译为目标语言
    Translate {
        text: String,
        #[arg(short, long, default_value = "英文")]
        to: String,
    },
}

// 3. 定义 API 返回的数据结构 
#[derive(Deserialize)]
struct ApiResponse {
    output: Output,
}

#[derive(Deserialize)]
struct Output {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

// 4. 核心网络请求函数
async fn call_llm(client: &Client, api_key: &str, model: &str, system: &str, user: &str) -> Result<String> {
    // 构建请求体
    let body = json!({
        "model": model,
        "input": {
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        },
        "parameters": {
            "result_format": "message"
        }
    });

    // 发送请求
    let response = client
        .post(ENDPOINT)
        .bearer_auth(api_key) // 自动添加 "Bearer " 前缀
        .json(&body)
        .send()
        .await
        .context("无法连接到 DashScope 服务器")?;

    // 检查 HTTP 状态码 (例如 401, 400, 500)
    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        // 如果 API 报错，直接抛出错误信息，包含状态码和返回内容
        anyhow::bail!("API 请求失败 [{}]: {}", status, err_text);
    }

    // 解析 JSON 数据
    let api_response: ApiResponse = response.json().await
        .context("API 返回的数据格式无法解析")?;

    // 提取文本内容
    let content = api_response
        .output
        .choices
        .into_iter()
        .next()
        .context("API 返回结果为空")?
        .message
        .content;

    Ok(content)
}

// 5. 主函数
#[tokio::main]
async fn main() -> Result<()> {
    // 加载 .env 环境变量
    dotenvy::dotenv().ok();

    // 获取 API Key
    let api_key = std::env::var("DASHSCOPE_API_KEY")
        .context("请在 .env 文件中设置 DASHSCOPE_API_KEY")?;

    // 解析命令行参数
    let cli = Cli::parse();
    
    // 创建一个复用的客户端
    let client = Client::new();

    // 根据子命令决定 system prompt 和 user input
    let (system_prompt, user_input) = match cli.command {
        Command::Ask { question } => (
            "你是一个乐于助人的 AI 助手。".to_string(),
            question,
        ),
        Command::Translate { text, to } => (
            format!("你是一个翻译助手，请将用户输入的内容翻译成{}，只输出译文，不要包含其他解释。", to),
            text,
        ),
    };

    // 调用 LLM 并打印结果
    println!("AI 正在思考...");
    let answer = call_llm(&client, &api_key, &cli.model, &system_prompt, &user_input).await?;
    println!("\n---\n{}", answer);

    Ok(())
}