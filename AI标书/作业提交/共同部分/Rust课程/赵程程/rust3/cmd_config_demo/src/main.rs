use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::fs;

// ====================== Part1：配置结构体 + 带错误处理的配置加载 ======================
#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
}

impl Config {
    /// 从 key=value 配置文件加载配置，全程 ? + context + bail!
    pub fn from_file(path: &str) -> Result<Self> {
        // 1. 读取文件，附加错误上下文
        let content =
            fs::read_to_string(path).with_context(|| format!("读取配置文件失败：{}", path))?;

        let mut api_key = None;
        let mut model = None;
        let mut max_tokens = None;

        // 2. 逐行解析 key=value
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, val) = line
                .split_once('=')
                .with_context(|| format!("配置行格式错误，缺少=号：{}", line))?;
            let key = key.trim();
            let val = val.trim();

            match key {
                "api_key" => api_key = Some(val.to_string()),
                "model" => model = Some(val.to_string()),
                "max_tokens" => {
                    let num = val
                        .parse::<u32>()
                        .with_context(|| format!("max_tokens 必须是合法数字，当前值：{}", val))?;
                    max_tokens = Some(num);
                }
                _ => bail!("存在未知配置项：{}", key),
            }
        }

        // 3. 校验字段完整性与合法性（bail! 主动抛错）
        let api_key = api_key.ok_or_else(|| anyhow!("缺失必填配置：api_key"))?;
        if api_key.is_empty() {
            bail!("api_key 不能为空字符串");
        }

        let model = model.ok_or_else(|| anyhow!("缺失必填配置：model"))?;
        if model.is_empty() {
            bail!("model 不能为空字符串");
        }

        let max_tokens = max_tokens.ok_or_else(|| anyhow!("缺失必填配置：max_tokens"))?;
        if max_tokens == 0 {
            bail!("max_tokens 必须大于 0");
        }

        Ok(Self {
            api_key,
            model,
            max_tokens,
        })
    }
}

// ====================== Part2：可扩展 Command 命令系统（Trait + Box<dyn> + 注册器） ======================
/// 命令特征：统一所有命令行为
pub trait Command {
    fn name(&self) -> &str;
    fn run(&self, args: &[String]) -> String;
}

// 命令1：Echo 命令 —— 拼接所有参数
#[derive(Default)]
pub struct EchoCommand;
impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn run(&self, args: &[String]) -> String {
        format!("Echo 输出：{}", args.join(" "))
    }
}

// 命令2：Uppercase 命令 —— 参数全部转大写
#[derive(Default)]
pub struct UppercaseCommand;
impl Command for UppercaseCommand {
    fn name(&self) -> &str {
        "upper"
    }

    fn run(&self, args: &[String]) -> String {
        let res: Vec<String> = args.iter().map(|s| s.to_uppercase()).collect();
        format!("大写输出：{}", res.join(" "))
    }
}

/// 命令注册器：存放所有动态命令
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    // 注册任意 Command 实现类
    pub fn register<C: Command + 'static>(&mut self, cmd: C) {
        self.commands.insert(cmd.name().to_string(), Box::new(cmd));
    }

    // 执行命令
    pub fn execute(&self, name: &str, args: &[String]) -> Result<String> {
        let cmd = self
            .commands
            .get(name)
            .with_context(|| format!("未找到命令：{}", name))?;
        Ok(cmd.run(args))
    }
}

// ====================== main 测试入口 ======================
fn main() -> Result<()> {
    // 1. 加载配置（带完整错误处理）
    let config = Config::from_file("config.txt")?;
    println!("✅ 配置加载成功：{:#?}\n", config);

    // 2. 注册所有命令
    let mut registry = CommandRegistry::new();
    registry.register(EchoCommand::default());
    registry.register(UppercaseCommand::default());

    // 3. 测试执行命令
    let echo_res = registry.execute("echo", &vec!["Hello".into(), "Rust".into()])?;
    println!("{}", echo_res);

    let upper_res = registry.execute("upper", &vec!["abc".into(), "123".into()])?;
    println!("{}", upper_res);

    Ok(())
}
