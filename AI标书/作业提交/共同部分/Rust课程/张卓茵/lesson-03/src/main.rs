use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs;

// 第一部分：配置解析（极简流水线版）

#[derive(Debug)]
struct Config {
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl Config {
    fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件：{path}"))?;

        // 用迭代器把文本直接变成 HashMap
        // 遇到格式错误或缺少等号的行，会立刻短路报错
        let values: HashMap<&str, &str> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| {
                line.split_once('=')
                    .with_context(|| format!("配置行格式错误（缺少 '=':{line}"))
            })
            .collect::<Result<_>>()?;

        // 提取并校验字段
        let api_key = values.get("api_key").context("缺少配置项: api_key")?;
        if api_key.is_empty() {
            bail!("api_key 不能为空");
        }

        let max_tokens = values
            .get("max_tokens")
            .context("缺少配置项: max_tokens")?
            .parse::<u32>()
            .context("max_tokens 必须是正整数")?;
        if max_tokens == 0 {
            bail!("max_tokens 必须大于 0");
        }

        Ok(Self {
            api_key: api_key.to_string(),
            model: values.get("model").unwrap_or(&"default-model").to_string(),
            max_tokens,
        })
    }
}
// 第二部分：Trait 与命令注册（接口与多态）

// 定义行为合约
trait Command {
    fn name(&self) -> &str;
    fn run(&self, args: &[String]) -> String;
}

// 具体的命令实现
struct EchoCommand;
struct UppercaseCommand;

impl Command for EchoCommand {
    fn name(&self) -> &str { "echo" }
    fn run(&self, args: &[String]) -> String { args.join(" ") }
}

impl Command for UppercaseCommand {
    fn name(&self) -> &str { "uppercase" }
    fn run(&self, args: &[String]) -> String { args.join(" ").to_uppercase() }
}

// 注册中心
struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    fn new() -> Self {
        Self { commands: HashMap::new() }
    }

    // 使用 impl Trait 接收，内部自动装箱
    fn register(&mut self, cmd: impl Command + 'static) {
        self.commands.insert(cmd.name().to_string(), Box::new(cmd));
    }

    fn execute(&self, name: &str, args: &[String]) -> Option<String> {
        self.commands.get(name).map(|cmd| cmd.run(args))
    }
}

// 第三部分：主程序
fn main() -> Result<()> {
    // 1. 加载配置
    println!("正在加载配置...");
    let config = Config::from_file("config.txt")?;
    println!(
        "✅ 配置加载成功：model={}, max_tokens={}, api_key_len={}\n",
        config.model,
        config.max_tokens,
        config.api_key.len(),
    );

    // 2. 初始化命令系统
    let mut registry = CommandRegistry::new();

    registry.register(EchoCommand);
    registry.register(UppercaseCommand);

    // 3. 测试执行
    let test_args = vec!["hello".to_string(), "rust".to_string()];

    if let Some(res) = registry.execute("echo", &test_args) {
        println!("👉 echo 执行结果: {}", res);
    }

    if let Some(res) = registry.execute("uppercase", &test_args) {
        println!("👉 uppercase 执行结果: {}", res);
    }

    Ok(())
}