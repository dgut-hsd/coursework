use futures::{Stream, TryStreamExt};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct ToolCallDelta {
    index: usize,
    function: FunctionDelta,
}

#[derive(Debug, Deserialize)]
struct FunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Default)]
struct ToolCallBuilder {
    name: Option<String>,
    arguments: String,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug)]
pub struct StreamOutput {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StreamError {
    Utf8Error,
    JsonError(String),
    InvalidToolCallArguments(String),
    StreamError(String),
}

pub struct StreamParser;

impl StreamParser {
    pub async fn parse_stream(
        mut stream: impl Stream<Item = Result<Vec<u8>, StreamError>> + Unpin,
    ) -> Result<StreamOutput, StreamError> {
        let mut text = String::new();
        let mut tool_calls: HashMap<usize, ToolCallBuilder> = HashMap::new();

        while let Some(bytes) = stream.try_next().await? {
            let line = String::from_utf8(bytes).map_err(|_| StreamError::Utf8Error)?;

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

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorInfo,
}

#[derive(Debug, Deserialize)]
struct ErrorInfo {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    

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
        let chunks = vec![
            Ok("{\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"content\":\" \"}}]}\n".as_bytes().to_vec()),
            Ok("{\"choices\":[{\"delta\":{\"content\":\"World\"}}]}\n".as_bytes().to_vec()),
            Ok("[DONE]\n".as_bytes().to_vec()),
        ];

        let output = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await.unwrap();
        assert_eq!(output.text, "Hello World");
        assert!(output.tool_calls.is_empty());
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
    }

    #[tokio::test]
    async fn test_parse_error_frame() {
        let chunks = vec![
            Ok("{\"error\":{\"message\":\"Rate limited\",\"type\":\"rate_limit\"}}\n".as_bytes().to_vec()),
        ];

        let result = StreamParser::parse_stream(TestStream { chunks, index: 0 }).await;
        assert!(matches!(result, Err(StreamError::StreamError(_))));
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
    }
}
