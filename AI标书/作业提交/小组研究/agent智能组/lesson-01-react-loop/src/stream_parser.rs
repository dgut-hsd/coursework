use futures::{Stream, TryStreamExt};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// 流式数据块 —— 从 SSE 流中解析出的单个数据块
#[derive(Debug, Deserialize)]
struct StreamChunk {
    /// 选择列表（通常只有一个选择）
    choices: Vec<Choice>,
}

/// 选择 —— 模型返回的一个选项
#[derive(Debug, Deserialize)]
struct Choice {
    /// 增量数据（每次返回的增量内容）
    delta: Delta,
}

/// 增量数据 —— 包含文本内容或工具调用的增量
#[derive(Debug, Deserialize)]
struct Delta {
    /// 文本内容（如果是文本输出）
    content: Option<String>,
    /// 工具调用列表（如果是工具调用输出）
    tool_calls: Option<Vec<ToolCallDelta>>,
}

/// 工具调用增量 —— 工具调用的增量信息
#[derive(Debug, Deserialize)]
struct ToolCallDelta {
    /// 工具调用的索引（支持同时调用多个工具）
    index: usize,
    /// 函数信息
    function: FunctionDelta,
}

/// 函数增量 —— 函数调用的增量信息
#[derive(Debug, Deserialize)]
struct FunctionDelta {
    /// 函数名称（工具名称）
    name: Option<String>,
    /// 函数参数（增量片段，需要拼接）
    arguments: Option<String>,
}

/// 工具调用构建器 —— 用于累积工具调用的增量数据
#[derive(Debug, Default)]
struct ToolCallBuilder {
    /// 工具名称
    name: Option<String>,
    /// 工具参数（累积拼接中）
    arguments: String,
}

/// 工具调用 —— 完整的工具调用信息
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// 工具名称
    pub name: String,
    /// 工具参数（完整的 JSON）
    pub arguments: Value,
}

/// 流式输出 —— 解析完成后的结果
#[derive(Debug)]
pub struct StreamOutput {
    /// 文本内容（所有文本增量拼接后的结果）
    pub text: String,
    /// 工具调用列表（所有工具调用解析后的结果）
    pub tool_calls: Vec<ToolCall>,
}

/// 流式解析错误类型
#[derive(Debug, PartialEq, Clone)]
pub enum StreamError {
    /// UTF-8 解码错误
    Utf8Error,
    /// JSON 解析错误
    JsonError(String),
    /// 工具调用参数格式错误
    InvalidToolCallArguments(String),
    /// 流错误（如 API 返回的错误）
    StreamError(String),
}

/// 流式解析器 —— 解析 SSE 流式数据
pub struct StreamParser;

impl StreamParser {
    /// 解析流式数据
    /// 参数：stream — SSE 数据流（每个元素是一行数据）
    /// 返回：解析后的文本和工具调用
    pub async fn parse_stream(
        mut stream: impl Stream<Item = Result<Vec<u8>, StreamError>> + Unpin,
    ) -> Result<StreamOutput, StreamError> {
        // 累积的文本内容
        let mut text = String::new();
        // 工具调用构建器：按索引累积（支持多个工具并行调用）
        let mut tool_calls: HashMap<usize, ToolCallBuilder> = HashMap::new();

        // 逐行读取流数据
        while let Some(bytes) = stream.try_next().await? {
            // 将字节转换为字符串
            let line = String::from_utf8(bytes).map_err(|_| StreamError::Utf8Error)?;

            // 遇到 [DONE] 结束符，停止解析
            if line.trim() == "[DONE]" {
                break;
            }

            // 提取数据部分（去掉 "data: " 前缀）
            let data_str = if line.starts_with("data: ") {
                &line[6..]
            } else if line.trim().is_empty() || line.starts_with("event:") || line.starts_with("retry:") {
                // 跳过空行、event 行、retry 行
                continue;
            } else {
                // 直接使用整行
                &line
            };

            // 检查是否是错误响应
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(data_str) {
                return Err(StreamError::StreamError(error.error.message));
            }

            // 解析 JSON 数据
            let data: StreamChunk = serde_json::from_str(data_str)
                .map_err(|e| StreamError::JsonError(e.to_string()))?;

            // 处理每个选择
            for choice in data.choices {
                // 如果是文本增量，追加到文本
                if let Some(content) = choice.delta.content {
                    text.push_str(&content);
                }
                // 如果是工具调用增量，累积到构建器
                if let Some(tc_deltas) = choice.delta.tool_calls {
                    for tc in tc_deltas {
                        // 获取或创建对应的工具调用构建器
                        let builder = tool_calls.entry(tc.index).or_default();
                        // 更新工具名称（如果有）
                        if let Some(name) = tc.function.name {
                            builder.name = Some(name);
                        }
                        // 追加参数片段（关键：参数是增量的，需要拼接）
                        if let Some(args) = tc.function.arguments {
                            builder.arguments.push_str(&args);
                        }
                    }
                }
            }
        }

        // 将所有工具调用构建器转换为完整的工具调用
        let tool_calls = tool_calls
            .into_values()
            .map(|b| {
                // 将拼接好的参数字符串解析为 JSON
                let args: Value = serde_json::from_str(&b.arguments)
                    .map_err(|e| StreamError::InvalidToolCallArguments(e.to_string()))?;
                Ok(ToolCall {
                    name: b.name.unwrap_or_default(),
                    arguments: args,
                })
            })
            .collect::<Result<Vec<_>, StreamError>>()?;

        Ok(StreamOutput { text, tool_calls })
    }

    /// 解析流式数据（字节版本）
    /// 与 parse_stream 类似，但输入是字节向量
    pub async fn parse_stream_bytes(
        mut stream: impl Stream<Item = Result<Vec<u8>, StreamError>> + Unpin,
    ) -> Result<StreamOutput, StreamError> {
        let mut text = String::new();
        let mut tool_calls: HashMap<usize, ToolCallBuilder> = HashMap::new();

        while let Some(bytes) = stream.try_next().await? {
            let line = String::from_utf8(bytes.to_vec()).map_err(|_| StreamError::Utf8Error)?;

            if line.trim() == "[DONE]" {
                break;
            }

            let data_str = if line.starts_with("data: ") {
                &line[6..]
            } else if line.trim().is_empty() || line.starts_with("event:") || line.starts_with("retry:") {
                continue;
            } else {
                &line
            };

            if let Ok(error) = serde_json::from_str::<ErrorResponse>(data_str) {
                return Err(StreamError::StreamError(error.error.message));
            }

            let data: StreamChunk = serde_json::from_str(data_str)
                .map_err(|e| StreamError::JsonError(e.to_string()))?;

            for choice in data.choices {
                if let Some(content) = choice.delta.content {
                    text.push_str(&content);
                }
                if let Some(tc_deltas) = choice.delta.tool_calls {
                    for tc in tc_deltas {
                        let builder = tool_calls.entry(tc.index).or_default();
                        if let Some(name) = tc.function.name {
                            builder.name = Some(name);
                        }
                        if let Some(args) = tc.function.arguments {
                            builder.arguments.push_str(&args);
                        }
                    }
                }
            }
        }

        let tool_calls = tool_calls
            .into_values()
            .map(|b| {
                let args: Value = serde_json::from_str(&b.arguments)
                    .map_err(|e| StreamError::InvalidToolCallArguments(e.to_string()))?;
                Ok(ToolCall {
                    name: b.name.unwrap_or_default(),
                    arguments: args,
                })
            })
            .collect::<Result<Vec<_>, StreamError>>()?;

        Ok(StreamOutput { text, tool_calls })
    }
}

/// 错误响应结构 —— API 返回的错误信息
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorInfo,
}

/// 错误信息结构
#[derive(Debug, Deserialize)]
struct ErrorInfo {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试用流 —— 模拟 SSE 数据流
    struct TestStream {
        chunks: Vec<Result<Vec<u8>, StreamError>>,
        index: usize,
    }

    impl Stream for TestStream {
        type Item = Result<Vec<u8>, StreamError>;

        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            if self.index >= self.chunks.len() {
                std::task::Poll::Ready(None)
            } else {
                let item = self.chunks[self.index].clone();
                self.index += 1;
                std::task::Poll::Ready(Some(item))
            }
        }
    }

    #[tokio::test]
    async fn test_parse_text_stream() {
        println!("===测试 流式解析器 ===");
        let chunks = vec![
            Ok("{\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"content\":\" \"}}]}\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"content\":\"World\"}}]}\n".as_bytes().to_vec()),
            Ok("[DONE]\n".as_bytes().to_vec()),
        ];

        let output = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await.unwrap();
        assert_eq!(output.text, "Hello World");
        assert!(output.tool_calls.is_empty());
        println!("✓ 解析纯文本流");
    }

    #[tokio::test]
    async fn test_parse_tool_call_stream() {
        let chunks = vec![
            Ok("{\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"search\",\"arguments\":\"{\\\"query\\\":\\\"test\\\"}\"}}]}}]}\n".as_bytes().to_vec()),
            Ok("[DONE]\n".as_bytes().to_vec()),
        ];

        let output = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await.unwrap();
        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].name, "search");
        println!("✓ 解析工具调用流");
    }

    #[tokio::test]
    async fn test_parse_tool_call_incremental() {
        let chunks = vec![
            Ok("{\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"retrieve\",\"arguments\":\"{\\\"id\\\":\"}}]}}]}\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"123}\"}}]}}]}\n".as_bytes().to_vec()),
            Ok("[DONE]\n".as_bytes().to_vec()),
        ];

        let output = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await.unwrap();
        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].name, "retrieve");
        println!("✓ 解析增量工具调用（参数分多帧发送）");
    }

    #[tokio::test]
    async fn test_parse_error_frame() {
        let chunks = vec![
            Ok("{\"error\":{\"message\":\"Rate limited\",\"type\":\"rate_limit\"}}\n".as_bytes().to_vec()),
        ];

        let result = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await;
        assert!(matches!(result, Err(StreamError::StreamError(_))));
        println!("✓ 解析错误帧");
    }

    #[tokio::test]
    async fn test_parse_ignores_non_data_lines() {
        let chunks = vec![
            Ok("event: message\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n".as_bytes().to_vec()),
            Ok("retry: 1000\n".as_bytes().to_vec()),
            Ok("[DONE]\n".as_bytes().to_vec()),
        ];

        let output = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await.unwrap();
        assert_eq!(output.text, "Hi");
        println!("✓ 忽略非数据行（event、retry 等）");
    }

    #[tokio::test]
    async fn test_parse_mixed_text_and_tool_call() {
        let chunks = vec![
            Ok("{\"choices\":[{\"delta\":{\"content\":\"Let me search for \"}}]}\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"search\",\"arguments\":\"{\\\"q\\\":\\\"test\\\"}\"}}]}}]}\n".as_bytes().to_vec()),
            Ok("[DONE]\n".as_bytes().to_vec()),
        ];

        let output = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await.unwrap();
        assert_eq!(output.text, "Let me search for ");
        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].name, "search");
        println!("✓ 解析混合文本和工具调用");
    }
}
