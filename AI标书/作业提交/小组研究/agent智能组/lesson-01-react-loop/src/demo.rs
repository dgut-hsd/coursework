use crate::token_budget::{TokenBudgetManager, MessageType, validate_token_count, DashScopeError};

pub async fn run_demo() {
    println!("=== Token Budget 管理器演示 ===");
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 1：Token Budget 管理器");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\n--- 步骤1：DashScope API Token计数验证 ---");
    let test_texts = vec![
        "你好",
        "你是一个有帮助的助手。",
        "建筑工程领域的法律合规审查需要注意哪些方面？",
        "根据《中华人民共和国建筑法》第十二条规定，从事建筑活动的建筑施工企业、勘察单位、设计单位和工程监理单位，应当具备下列条件：有符合国家规定的注册资本；有与其从事的建筑活动相适应的具有法定执业资格的专业技术人员；有从事相关建筑活动所应有的技术装备；法律、行政法规规定的其他条件。",
    ];
    
    for text in test_texts {
        match validate_token_count(text).await {
            Ok((local, api, error)) => {
                println!("文本: \"{}\"", text);
                println!("  本地估算: {} Token", local);
                println!("  API返回: {} Token", api);
                println!("  误差: {:.2}%", error);
                if error < 10.0 {
                    println!("  ✓ 误差在10%以内，验证通过");
                } else {
                    println!("  ⚠️ 误差超过10%，建议优化");
                }
            }
            Err(e) => {
                match e {
                    DashScopeError::MissingApiKey => {
                        println!("⚠️ 未设置 DASHSCOPE_API_KEY 环境变量");
                        println!("  创建 .env 文件并添加: DASHSCOPE_API_KEY=your_api_key");
                        println!("  跳过 API 验证，使用本地估算");
                    }
                    _ => {
                        println!("⚠️ API 验证失败: {:?}", e);
                        println!("  跳过 API 验证，使用本地估算");
                    }
                }
            }
        }
        println!();
    }
    
    println!("\n--- 步骤2：模拟长对话（50轮）验证裁剪策略 ---");
    
    // 创建一个只有 200 Token 的预算（很小，容易触发裁剪）
    let system_prompt = "你是一个有帮助的助手。";
    let mut manager = TokenBudgetManager::new(200, system_prompt);
    
    println!("初始状态:");
    println!("  System Prompt: \"{}\"", system_prompt);
    println!("  总预算: {} Token", 200);
    println!("  已使用: {} Token", manager.used());
    println!("  消息数: {}", manager.message_count());
    println!();
    
    // 添加 50 轮对话
    for i in 0..50 {
        let user_msg = format!("用户消息 {}", i);
        let assistant_msg = format!("助手回复 {}：这是一个很长很长的回复，用来消耗更多的 Token。", i);
        
        match manager.add_message(MessageType::User, &user_msg) {
            Ok(_) => {},
            Err(_) => {
                println!("⚠️ 第 {} 轮：添加用户消息失败（Token不足）", i);
                break;
            }
        }
        
        match manager.add_message(MessageType::Assistant, &assistant_msg) {
            Ok(_) => {},
            Err(_) => {
                println!("⚠️ 第 {} 轮：添加助手回复失败（Token不足）", i);
                break;
            }
        }
        
        // 每10轮打印一次状态
        if (i + 1) % 10 == 0 {
            println!("第 {} 轮对话后:", i + 1);
            println!("  已使用: {} Token", manager.used());
            println!("  剩余: {} Token", manager.remaining());
            println!("  消息数: {}", manager.message_count());
            println!("  System Prompt 仍在: {}", manager.has_system_prompt());
            println!();
        }
    }
    
    // 最终状态
    println!("=== 最终状态 ===");
    println!("  已使用: {} Token", manager.used());
    println!("  剩余: {} Token", manager.remaining());
    println!("  消息数: {}", manager.message_count());
    println!("  System Prompt 仍在: {}", manager.has_system_prompt());
    
    // 查看消息内容
    println!("\n=== 剩余消息内容 ===");
    for (i, msg) in manager.messages().iter().enumerate() {
        let msg_type = match msg.msg_type {
            MessageType::System => "【系统】",
            MessageType::User => "【用户】",
            MessageType::Assistant => "【助手】",
            MessageType::ToolResult => "【工具结果】",
            MessageType::ToolCall => "【工具调用】",
        };
        println!("  {} [{} Token]: {}", i + 1, msg.tokens, msg.content);
    }
    
    println!("\n✓ Token Budget 管理器演示完成");
}
