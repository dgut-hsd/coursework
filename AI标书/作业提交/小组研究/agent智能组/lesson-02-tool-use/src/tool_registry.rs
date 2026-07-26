use chrono;
use serde_json::Value;
use std::collections::HashMap;

pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn definition(&self) -> Value;
    fn execute(&self, args: Value) -> tokio::task::JoinHandle<Result<Value, ToolError>>;
}

#[derive(Debug)]
pub enum ToolError {
    InvalidArguments(String),
    ExecutionError(String),
    ToolNotFound(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::InvalidArguments(s) => write!(f, "Invalid arguments: {}", s),
            ToolError::ExecutionError(s) => write!(f, "Execution error: {}", s),
            ToolError::ToolNotFound(s) => write!(f, "Tool not found: {}", s),
        }
    }
}

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn AgentTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn AgentTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| {
                let def = tool.definition();
                ToolDefinition {
                    name: tool.name().to_string(),
                    description: def.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    parameters: def.get("parameters")
                        .cloned()
                        .unwrap_or(Value::Object(serde_json::Map::new())),
                }
            })
            .collect()
    }

    pub async fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or(ToolError::ToolNotFound(name.to_string()))?;
        tool.execute(args).await.map_err(|e| ToolError::ExecutionError(format!("{}", e)))?
    }

    pub fn get_tool(&self, name: &str) -> Option<&dyn AgentTool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

pub struct SearchKnowledgeTool;

impl AgentTool for SearchKnowledgeTool {
    fn name(&self) -> &str {
        "search_knowledge"
    }

    fn definition(&self) -> Value {
        serde_json::json!({
            "name": "search_knowledge",
            "description": "Search for legal knowledge and regulations",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query string"
                    }
                },
                "required": ["query"]
            }
        })
    }

    fn execute(&self, args: Value) -> tokio::task::JoinHandle<Result<Value, ToolError>> {
        tokio::spawn(async move {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .ok_or(ToolError::InvalidArguments("query is required".to_string()))?;

            let results = vec![
                serde_json::json!({
                    "law_id": "law_001",
                    "title": "建筑法",
                    "article": "第12条",
                    "text": format!("从事建筑活动的建筑施工企业、勘察单位、设计单位和工程监理单位，应当具备下列条件：（一）有符合国家规定的注册资本；（二）有与其从事的建筑活动相适应的具有法定执业资格的专业技术人员；（三）有从事相关建筑活动所应有的技术装备；（四）法律、行政法规规定的其他条件。"),
                    "relevance": 0.95
                }),
                serde_json::json!({
                    "law_id": "law_002",
                    "title": "招标投标法",
                    "article": "第26条",
                    "text": format!("投标人应当具备承担招标项目的能力；国家有关规定对投标人资格条件或者招标文件对投标人资格条件有规定的，投标人应当具备规定的资格条件。"),
                    "relevance": 0.88
                })
            ];

            Ok(serde_json::json!({
                "query": query,
                "results": results,
                "total": results.len()
            }))
        })
    }
}

pub struct ReadSectionTool;

impl AgentTool for ReadSectionTool {
    fn name(&self) -> &str {
        "read_section"
    }

    fn definition(&self) -> Value {
        serde_json::json!({
            "name": "read_section",
            "description": "Read a specific section from a document",
            "parameters": {
                "type": "object",
                "properties": {
                    "clause_id": {
                        "type": "string",
                        "description": "Clause ID to read"
                    }
                },
                "required": ["clause_id"]
            }
        })
    }

    fn execute(&self, args: Value) -> tokio::task::JoinHandle<Result<Value, ToolError>> {
        tokio::spawn(async move {
            let clause_id = args.get("clause_id")
                .and_then(|v| v.as_str())
                .ok_or(ToolError::InvalidArguments("clause_id is required".to_string()))?;

            let mock_sections: HashMap<&str, &str> = [
                ("cl_001", "第一条 为了规范招标投标活动，保护国家利益、社会公共利益和招标投标活动当事人的合法权益，提高经济效益，保证项目质量，制定本法。"),
                ("cl_002", "第二条 在中华人民共和国境内进行招标投标活动，适用本法。"),
                ("cl_042", "第四十二条 评标委员会经评审，认为所有投标都不符合招标文件要求的，可以否决所有投标。"),
                ("cl_087", "第八十七条 招标代理机构违反本法规定，泄露应当保密的与招标投标活动有关的情况和资料的，或者与招标人、投标人串通损害国家利益、社会公共利益或者他人合法权益的，处五万元以上二十五万元以下的罚款"),
            ].into();

            let content = mock_sections.get(clause_id).copied().unwrap_or("Clause not found");

            Ok(serde_json::json!({
                "clause_id": clause_id,
                "content": content
            }))
        })
    }
}

pub struct OutputFindingTool;

impl AgentTool for OutputFindingTool {
    fn name(&self) -> &str {
        "output_finding"
    }

    fn definition(&self) -> Value {
        serde_json::json!({
            "name": "output_finding",
            "description": "Output final audit finding and conclusion",
            "parameters": {
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "Summary of findings"
                    },
                    "issues": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "clause_id": {"type": "string"},
                                "description": {"type": "string"},
                                "severity": {"type": "string"},
                                "law_ref": {"type": "string"}
                            }
                        }
                    },
                    "compliant": {"type": "boolean"}
                },
                "required": ["summary", "compliant"]
            }
        })
    }

    fn execute(&self, args: Value) -> tokio::task::JoinHandle<Result<Value, ToolError>> {
        tokio::spawn(async move {
            let summary = args.get("summary")
                .and_then(|v| v.as_str())
                .ok_or(ToolError::InvalidArguments("summary is required".to_string()))?;

            let compliant = args.get("compliant")
                .and_then(|v| v.as_bool())
                .ok_or(ToolError::InvalidArguments("compliant is required".to_string()))?;

            Ok(serde_json::json!({
                "summary": summary,
                "compliant": compliant,
                "issues": args.get("issues").cloned().unwrap_or(Value::Array(Vec::new())),
                "timestamp": chrono::Local::now().to_rfc3339()
            }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_registry_register() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(SearchKnowledgeTool));
        registry.register(Box::new(ReadSectionTool));
        registry.register(Box::new(OutputFindingTool));

        assert_eq!(registry.tool_names().len(), 3);
        assert!(registry.tool_names().contains(&"search_knowledge".to_string()));
        assert!(registry.tool_names().contains(&"read_section".to_string()));
        assert!(registry.tool_names().contains(&"output_finding".to_string()));
    }

    #[tokio::test]
    async fn test_tool_registry_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(SearchKnowledgeTool));

        let defs = registry.definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "search_knowledge");
    }

    #[tokio::test]
    async fn test_search_knowledge_execute() {
        let tool = SearchKnowledgeTool;
        let args = serde_json::json!({"query": "建筑资质"});
        let result = tool.execute(args).await.unwrap().unwrap();

        assert!(result.get("query").is_some());
        assert!(result.get("results").is_some());
    }

    #[tokio::test]
    async fn test_read_section_execute() {
        let tool = ReadSectionTool;
        let args = serde_json::json!({"clause_id": "cl_042"});
        let result = tool.execute(args).await.unwrap().unwrap();

        assert_eq!(result.get("clause_id").and_then(|v| v.as_str()), Some("cl_042"));
        assert!(result.get("content").is_some());
    }

    #[tokio::test]
    async fn test_output_finding_execute() {
        let tool = OutputFindingTool;
        let args = serde_json::json!({
            "summary": "Test finding",
            "compliant": true
        });
        let result = tool.execute(args).await.unwrap().unwrap();

        assert_eq!(result.get("summary").and_then(|v| v.as_str()), Some("Test finding"));
        assert_eq!(result.get("compliant").and_then(|v| v.as_bool()), Some(true));
    }
}
