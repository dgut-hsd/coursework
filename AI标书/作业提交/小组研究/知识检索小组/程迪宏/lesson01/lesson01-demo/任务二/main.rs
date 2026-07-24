/// 一、导包
/// 1.anyhow 用于错误处理
use anyhow::{Context, Result};
/// 2.ndarray 用于数组操作
use ndarray::{Array2, Array3, Axis, s};
/// 3.ort 用于ONNX模型推理
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Tensor;
use ort::memory::{AllocationDevice, AllocatorType, MemoryInfo, MemoryType};
/// 4.std 用于线程安全的内存分配
use std::sync::{Arc, Mutex};
use std::fs;
/// 5.tokenizers 用于文本分词
use tokenizers::Tokenizer;

///二、数据结构

/// 1.常量

/// 环境变量名：控制每次 ONNX 推理的 batch 大小
const ENV_BATCH_SIZE: &str = "EMBEDDING_BATCH_SIZE";
const DEFAULT_BATCH_SIZE: usize = 32;

/// BGE-M3 输出维度
const HIDDEN_DIM: usize = 1024;

/// 2.结构体
///
/// 概念理解：
/// 1.Send + Sync：所有权（Send）和引用（Sync）可以安全地跨线程传递
/// 2.Arc::clone() 是廉价操作 — 它只增加原子计数，不拷贝内部数据。
/// 3.Arc<Mutex<T>> 模式：用于在线程安全地共享可变状态，如 ONNX 会话。

/// 内层结构体：包含模型推理的所有必要信息
struct EmbeddingEngineInner {
    tokenizer: Tokenizer,       // 文本分词器
    session: Mutex<Session>,    // ONNX模型会话
    input_names: Vec<String>,   // 输入节点名称
    output_name: String,        // 输出节点名称
    batch_size: usize,          // 批次大小
}
/// 外层结构体：用于线程安全的模型推理，由于Arc::clone() 是廉价操作，可以安全地克隆，所以这里使用 Arc 来管理模型会话
#[derive(Clone)]
pub struct EmbeddingEngine {
    inner: Arc<EmbeddingEngineInner>,
}

/// 任务 2：文本对结构体，包含两个文本和人工标注的相似度
struct TextPair {
    a: String,
    b: String,
    human_score: f32,  // 0.0 ~ 1.0
}

/// 三、加载模型 + Tokenizer
/// 1.加载模型：从 ONNX 模型文件加载模型，初始化 ONNX 会话
/// 2.加载 Tokenizer：从配置文件加载 Tokenizer 配置，初始化 Tokenizer
impl EmbeddingEngine {
    /// model文件内容：ONNX模型文件 + tokenizer.json
    pub fn load(model_dir: &str) -> Result<Self> {
        //1. 文件路径
        let model_path = format!("{}/model.onnx", model_dir);
        let tokenizer_path = format!("{}/tokenizer.json", model_dir);

        // 1.1 加载 Tokenizer
        // Tokenizer::from_file 的 error type 是 Box<dyn Error>，和 anyhow 不兼容，用 map_err 转
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("加载 tokenizer 失败 {}: {}", tokenizer_path, e))?;

        //1.2 加载ONNX模型
         let session = Session::builder()   // 创建 ONNX 会话构建器
            .map_err(|e| anyhow::anyhow!("创建 session builder 失败: {e}"))? //由于返回值是ort::Error<SessionBuilder>，用.map_err()手动转成 anyhow::anyhow 处理错误
            .with_optimization_level(GraphOptimizationLevel::Level2) // 设置优化级别，Level2 最适合Transformer 模型
            .map_err(|e| anyhow::anyhow!("设置优化级别失败: {e}"))?
            .commit_from_file(&model_path)
            .map_err(|e| anyhow::anyhow!("加载 ONNX 模型失败 {}: {}", model_path, e))?;
    
        // 1.3 通过ONNX模型获取输入节点和输出节点名称
        let input_names: Vec<String> = session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        let output_name = session
            .outputs()
            .first()
            .map(|o| o.name().to_string())
            .context("模型没有任何输出")?;
        //打印输入节点和输出节点名称
        println!("[load] ONNX 输入: {:?}", input_names);
        println!("[load] ONNX 输出: {}", output_name);


        // 1.4 从环境变量读取批次大小batch_size
        let batch_size = std::env::var(ENV_BATCH_SIZE)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_BATCH_SIZE);

        Ok(EmbeddingEngine {
            inner: Arc::new(EmbeddingEngineInner {
                tokenizer,
                session: Mutex::new(session),
                input_names,
                output_name,
                batch_size,
            }),
        })
    }
}

/// 四、产出模型的三个输入矩阵：
/// 1.input_ids: 输入的 token ID 序列
/// 2.attention_mask: 注意力掩码，用于表示哪些 token 是真实 token，哪些是填充 token
/// 3.token_type_ids: 用于表示不同文本（如对话中的用户和助手）的 token 类型
impl EmbeddingEngineInner {
    fn tokenize_batch(&self, texts: &[&str]) -> Result<TokenizedBatch> {
        // 1.从 tokenizer 配置中读 pad_token_id
        let pad_id = self
            .tokenizer
            .get_padding()
            .map(|p| p.pad_id)
            .unwrap_or(0); // 如果 tokenizer 没有配置 padding，使用 0 作为填充

        // 2.遍历文本：对每个文本进行编码，记录最大长度
        let mut all_ids: Vec<Vec<i64>> = Vec::with_capacity(texts.len());
        let mut max_len = 0usize;

        for text in texts {
            let encoding = self
                .tokenizer
                .encode(*text, true) // 把文本切成 token ID 序列。true 表示添加特殊 token如[CLS]、[SEP]等
                .map_err(|e| anyhow::anyhow!("tokenize 失败：{}", e))?;

            // 转换为 i64 类型（模型输入要求）
            let ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
            // 记录最大长度
            max_len = max_len.max(ids.len());
            all_ids.push(ids); 
        }

        // 构造 padded 矩阵 + attention_mask
        let batch = texts.len();
        let mut input_ids = Array2::<i64>::from_elem((batch, max_len), pad_id as i64);
        let mut attention_mask = Array2::<i64>::zeros((batch, max_len));

        // 填充真实数据
        for (i, ids) in all_ids.iter().enumerate() {
            for (j, &id) in ids.iter().enumerate() {
                input_ids[[i, j]] = id;
                attention_mask[[i, j]] = 1; // 真实 token → 1
            }
            // 剩余位置已在 from_elem 时初始化为 pad_id，attention_mask 保持 0
        }

        let token_type_ids = Array2::<i64>::zeros((batch, max_len));

        Ok(TokenizedBatch {
            input_ids,
            attention_mask,
            token_type_ids,
        })
    }
}
/// tokenize_batch 的返回值，包含模型输入的三个矩阵
struct TokenizedBatch {
    input_ids: Array2<i64>,
    attention_mask: Array2<i64>,
    token_type_ids: Array2<i64>,
}

/// 五、将ONNX输出的三维向量转化为句子向量（二维）
///
fn mean_pooling(
    hidden_states: &Array3<f32>,   // [batch, seq_len, 1024]
    attention_mask: &Array2<i64>,  // [batch, seq_len]
) -> Array2<f32> {
    let (batch, _seq_len, hidden_dim) = hidden_states.dim();

    // mask → f32 并扩维：[batch, seq_len] → [batch, seq_len, 1]
    // insert_axis 会 consume self，所以先 clone 一份用于后面的 count 计算
    let mask_f32 = attention_mask.mapv(|v| v as f32);
    let count_1d = mask_f32.sum_axis(Axis(1));  // [batch] — 有效 token 数
    let mask_3d = mask_f32.insert_axis(Axis(2)); // [batch, seq_len, 1]

    // padding 位置乘以 0，不参与求和
    let masked = hidden_states * &mask_3d;

    // 沿 seq_len 维求和 → [batch, 1024]
    let sum_2d = masked.sum_axis(Axis(1));

    // 逐行 Mean + L2 Normalize
    let mut result = Array2::<f32>::zeros((batch, hidden_dim));
    for b in 0..batch {
        let cnt = count_1d[[b]];
        if cnt == 0.0 {
            continue;
        }

        // Mean：取第 b 行的 sum，除以 count
        let row = sum_2d.slice(s![b, ..]);
        let mean: Vec<f32> = row.iter().map(|&v| v / cnt).collect();

        // L2 Normalize
        let norm_sq: f32 = mean.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm > 0.0 {
            for (j, &v) in mean.iter().enumerate() {
                result[[b, j]] = v / norm;
            }
        }
    }

    result
}

/// 五、 run_onnx_batch() — ONNX IO Binding 推理
/// 
/// 分配器（Allocator）在run_onnx_batch() 方法中使用，用于在 ONNX 模型中推理时分配内存
/// 1.Session级别分配器：每个 ONNX 会话都有一个分配器，用于在会话中分配内存。一次设置,后续所有绑定都使用这个分配器
/// 2.Binding级别分配器：每个绑定（Binding）都有一个分配器，用于在绑定中分配内存。更灵活不同推理可用不同的分配策略（当前代码所用）
impl EmbeddingEngineInner {
    fn run_onnx_batch(
        &self,
        input_ids: &Array2<i64>,
        attention_mask: &Array2<i64>,
        token_type_ids: &Array2<i64>,
    ) -> Result<Array2<f32>> {
        let batch = input_ids.dim().0;
        let seq_len = input_ids.dim().1;

        // ── 1. ndarray → ort::Tensor（零拷贝） ──
        let input_ids_val = Tensor::from_array(input_ids.clone())
            .context("创建 input_ids tensor 失败")?;
        let attention_mask_val = Tensor::from_array(attention_mask.clone())
            .context("创建 attention_mask tensor 失败")?;
        let token_type_ids_val = Tensor::from_array(token_type_ids.clone())
            .context("创建 token_type_ids tensor 失败")?;

        // ── 2. IO Binding ──
        let mut binding = self.session.lock().unwrap().create_binding()?;

        binding.bind_input(&self.input_names[0], &input_ids_val)?;
        binding.bind_input(&self.input_names[1], &attention_mask_val)?;
        if self.input_names.len() > 2 {
            binding.bind_input(&self.input_names[2], &token_type_ids_val)?;
        }

        // ── 3. 绑定输出（让 runtime 自动分配，不预定义 shape） ──
        // 不同导出的模型输出形状不同：
        //   - 原始 ONNX: [batch, seq_len, 1024]（3D hidden states）
        //   - 量化/sentence-similarity 模型: [batch, 1024]（已池化）
        // bind_output_to_device 不预定义 shape，runtime 自动适配
        let output_memory = MemoryInfo::new(
            AllocationDevice::CPU,
            0,
            AllocatorType::Arena,
            MemoryType::CPUOutput,
        )?;
        binding.bind_output_to_device(&self.output_name, &output_memory)?;

        // ── 4. 执行推理 ──
        let mut session = self.session.lock().unwrap();
        let mut outputs = session
            .run_binding(&binding)
            .context("ONNX 推理失败")?;

        let output = outputs
            .remove(&self.output_name)
            .context("输出中找不到结果")?;
        let output_view: ndarray::ArrayViewD<f32> = output
            .try_extract_array::<f32>()
            .context("提取 ONNX 输出失败")?;

        // ── 5. 根据输出 rank 决定是否需要 mean pooling ──
        match output_view.ndim() {
            3 => {
                // 原始 hidden states [batch, seq_len, 1024] → mean_pooling → [batch, 1024]
                let hidden = output_view
                    .into_shape_with_order((batch, seq_len, HIDDEN_DIM))
                    .map_err(|e| anyhow::anyhow!("reshape 3D 输出失败: {}", e))?
                    .to_owned();
                Ok(mean_pooling(&hidden, attention_mask))
            }
            2 => {
                // 模型已内置池化 [batch, 1024] → 只做 L2 normalize
                let embeddings = output_view
                    .into_shape_with_order((batch, HIDDEN_DIM))
                    .map_err(|e| anyhow::anyhow!("reshape 2D 输出失败: {}", e))?;

                let mut result = Array2::<f32>::zeros((batch, HIDDEN_DIM));
                for b in 0..batch {
                    let row = embeddings.slice(s![b, ..]);
                    let norm_sq: f32 = row.iter().map(|x| x * x).sum();
                    let norm = norm_sq.sqrt();
                    if norm > 0.0 {
                        for (j, &v) in row.iter().enumerate() {
                            result[[b, j]] = v / norm;
                        }
                    }
                }
                Ok(result)
            }
            n => Err(anyhow::anyhow!(
                "不支持的输出维度: {} (期望 2 或 3)",
                n
            )),
        }
    }
}

// 六、 embed_batch() — 入口方法

//
// 对外接口。做三件事：
//   1. 把 texts 按 batch_size 切分
//   2. 对每个 sub-batch：tokenize → ONNX 推理 → mean_pooling
//   3. 合并所有结果

impl EmbeddingEngine {
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<[f32; HIDDEN_DIM]>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let inner = &self.inner;
        let mut all_embeddings: Vec<[f32; HIDDEN_DIM]> =
            Vec::with_capacity(texts.len());

        // ── 按 batch_size 切分成 sub-batch ──
        for chunk in texts.chunks(inner.batch_size) {
            // 1. Tokenize + dynamic padding
            let tokenized = inner.tokenize_batch(chunk)?;

            // 2. ONNX 推理 → 返回已 L2-normalized 的 [batch, 1024]
            let embeddings = inner.run_onnx_batch(
                &tokenized.input_ids,
                &tokenized.attention_mask,
                &tokenized.token_type_ids,
            )?;

            // 3. Array2<f32> → Vec<[f32; 1024]>
            for b in 0..embeddings.dim().0 {
                let slice = embeddings.slice(s![b, ..]);
                let mut arr = [0.0f32; HIDDEN_DIM];
                for (j, &v) in slice.iter().enumerate() {
                    arr[j] = v;
                }
                all_embeddings.push(arr);
            }
        }

        Ok(all_embeddings)
    }
}

// 七、 main() — 任务 2：验证语义表征能力（Spearman 相关系数）
fn main() -> Result<()> {
    let model_dir = std::env::args()
        .nth(1)
        .context("用法: cargo run -- <模型目录路径>\n模型目录下需包含 model.onnx 和 tokenizer.json")?;

    println!("=== 加载模型 ===");
    let engine = EmbeddingEngine::load(&model_dir)?;

    // ── 加载数据集 ──
    println!("\n=== 加载数据集 ===");
    let pairs = parse_dataset("dataset.txt")
        .context("请确保 dataset.txt 在当前目录下")?;
    println!("加载 {} 对标书文本对\n", pairs.len());

    // ── 计算每对的 cosine 相似度 ──
    let mut cosines = Vec::with_capacity(pairs.len());
    let mut human_scores = Vec::with_capacity(pairs.len());

    for (i, pair) in pairs.iter().enumerate() {
        // embed_batch 接受 &[&str]，传入单条文本
        let emb_a = engine.embed_batch(&[&pair.a])?;
        let emb_b = engine.embed_batch(&[&pair.b])?;
        let cos = cosine(&emb_a[0], &emb_b[0]);

        cosines.push(cos);
        human_scores.push(pair.human_score);

        // 打印部分结果作为参考
        if i < 5 || i % 20 == 19 {
            println!(
                "  [{:>3}] human={:.2}, cos={:.4}  {}",
                i + 1,
                pair.human_score,
                cos,
                if (cos - pair.human_score).abs() < 0.15 { "✓" } else { "" },
            );
        }
    }

    // ── 计算 Spearman 相关系数 ──
    let rho = spearman_rho(&cosines, &human_scores);

    println!("\n═══════════════════════════════════════");
    println!("  Spearman ρ = {:.4}", rho);
    println!("  目标：ρ > 0.80");
    if rho > 0.80 {
        println!("  ✅ 达到目标！模型语义表征能力合格。");
    } else {
        println!("  ❌ 未达到目标，请检查：");
        println!("      - 人工标注是否一致");
        println!("      - 文本对是否覆盖足够的难度梯度");
        println!("      - 模型是否正确加载");
    }
    println!("═══════════════════════════════════════");

    Ok(())
}

fn cosine(a: &[f32; HIDDEN_DIM], b: &[f32; HIDDEN_DIM]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na > 0.0 && nb > 0.0 {
        dot / (na * nb)
    } else {
        0.0
    }
}

/// 解析数据集文件，每行格式：id.text_a|||text_b|||human_score
fn parse_dataset(path: &str) -> Result<Vec<TextPair>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("读取数据集文件失败: {}", path))?;
    let mut pairs = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split("|||").collect();
        if parts.len() != 3 {
            continue;
        }

        // 解析 text_a：去掉开头的 "id."
        let text_a_part = parts[0].trim();
        let dot_pos = text_a_part.find('.').unwrap_or(0);
        let text_a = text_a_part[dot_pos + 1..].to_string();

        let text_b = parts[1].trim().to_string();
        let human_score: f32 = parts[2].trim().parse()
            .with_context(|| format!("解析分数失败: {}", parts[2]))?;

        pairs.push(TextPair { a: text_a, b: text_b, human_score });
    }

    Ok(pairs)
}

/// 计算 Spearman 等级相关系数
/// 先对两组数据分别排名，再计算排名间的 Pearson 相关系数
fn spearman_rho(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len();
    if n == 0 {
        return 0.0;
    }

    // 对一组值排名（处理并列情况：取平均排名）
    let rank = |vals: &[f32]| -> Vec<f32> {
        let n = vals.len();
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|&i, &j| vals[i].partial_cmp(&vals[j]).unwrap());

        let mut ranks = vec![0.0f32; n];
        let mut i = 0;
        while i < n {
            let mut j = i;
            while j < n && vals[indices[j]] == vals[indices[i]] {
                j += 1;
            }
            // 并列值的排名取平均
            let avg_rank = (i + j + 1) as f32 / 2.0;  // (i+1 + j) / 2
            for k in i..j {
                ranks[indices[k]] = avg_rank;
            }
            i = j;
        }
        ranks
    };

    let rx = rank(x);
    let ry = rank(y);

    // 对排名计算 Pearson 相关系数
    let mean_x = rx.iter().sum::<f32>() / n as f32;
    let mean_y = ry.iter().sum::<f32>() / n as f32;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = rx[i] - mean_x;
        let dy = ry[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        0.0
    } else {
        cov / (var_x.sqrt() * var_y.sqrt())
    }
}

