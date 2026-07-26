use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::agent_bus::{AgentBus, AuditFinding};
use crate::session_graph::{SessionGraph, AgentStatus, RelationType};

#[derive(Debug, Clone)]
pub struct DocumentProfile {
    pub doc_type: String,
    pub project_type: String,
    pub complexity: f64,
}

#[derive(Debug, Clone)]
pub struct CoordinatorDefinition {
    pub agent_mapping: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub document_id: String,
    pub findings: Vec<AuditFinding>,
    pub compliant: bool,
    pub summary: String,
    pub agent_stats: AgentStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_agents: usize,
    pub completed_agents: usize,
    pub failed_agents: usize,
    pub total_time_ms: u64,
}

pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn ReviewAgent>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent_type: String, agent: Box<dyn ReviewAgent>) {
        self.agents.insert(agent_type, agent);
    }

    pub fn get_agent(&self, agent_type: &str) -> Option<&dyn ReviewAgent> {
        self.agents.get(agent_type).map(|a| a.as_ref())
    }
}

pub trait ReviewAgent: Send + Sync {
    fn agent_type(&self) -> &str;
    fn review(&self, document: &Document, bus: &AgentBus) -> tokio::task::JoinHandle<Vec<AuditFinding>>;
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub clauses: HashMap<String, String>,
}

pub struct JudgeAgent;

impl ReviewAgent for JudgeAgent {
    fn agent_type(&self) -> &str {
        "JudgeAgent"
    }

    fn review(&self, document: &Document, bus: &AgentBus) -> tokio::task::JoinHandle<Vec<AuditFinding>> {
        let document = document.clone();
        let bus = bus.clone();
        
        tokio::spawn(async move {
            bus.broadcast(crate::agent_bus::AgentMessage::ReviewProgress {
                agent: "JudgeAgent".to_string(),
                progress: 50.0,
                stage: "analyzing".to_string(),
            });

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            bus.broadcast(crate::agent_bus::AgentMessage::ReviewProgress {
                agent: "JudgeAgent".to_string(),
                progress: 100.0,
                stage: "completed".to_string(),
            });

            vec![
                AuditFinding {
                    id: "judge_001".to_string(),
                    clause_id: "cl_001".to_string(),
                    description: "招标程序符合基本要求".to_string(),
                    severity: "low".to_string(),
                    law_ref: Some("law_001".to_string()),
                },
                AuditFinding {
                    id: "judge_002".to_string(),
                    clause_id: "cl_042".to_string(),
                    description: "否决投标条款符合法规".to_string(),
                    severity: "low".to_string(),
                    law_ref: Some("law_002".to_string()),
                },
            ]
        })
    }
}

pub struct LegalVerifyAgent;

impl ReviewAgent for LegalVerifyAgent {
    fn agent_type(&self) -> &str {
        "LegalVerifyAgent"
    }

    fn review(&self, document: &Document, bus: &AgentBus) -> tokio::task::JoinHandle<Vec<AuditFinding>> {
        let document = document.clone();
        let bus = bus.clone();
        
        tokio::spawn(async move {
            bus.broadcast(crate::agent_bus::AgentMessage::ReviewProgress {
                agent: "LegalVerifyAgent".to_string(),
                progress: 30.0,
                stage: "verifying".to_string(),
            });

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;

            bus.broadcast(crate::agent_bus::AgentMessage::ReviewProgress {
                agent: "LegalVerifyAgent".to_string(),
                progress: 100.0,
                stage: "completed".to_string(),
            });

            vec![
                AuditFinding {
                    id: "legal_001".to_string(),
                    clause_id: "cl_087".to_string(),
                    description: "罚款金额符合法定范围".to_string(),
                    severity: "low".to_string(),
                    law_ref: Some("law_003".to_string()),
                },
            ]
        })
    }
}

pub struct ComplianceAgent;

impl ReviewAgent for ComplianceAgent {
    fn agent_type(&self) -> &str {
        "ComplianceAgent"
    }

    fn review(&self, document: &Document, bus: &AgentBus) -> tokio::task::JoinHandle<Vec<AuditFinding>> {
        let document = document.clone();
        let bus = bus.clone();
        
        tokio::spawn(async move {
            bus.broadcast(crate::agent_bus::AgentMessage::ReviewProgress {
                agent: "ComplianceAgent".to_string(),
                progress: 40.0,
                stage: "checking".to_string(),
            });

            tokio::time::sleep(std::time::Duration::from_millis(250)).await;

            bus.broadcast(crate::agent_bus::AgentMessage::ReviewProgress {
                agent: "ComplianceAgent".to_string(),
                progress: 100.0,
                stage: "completed".to_string(),
            });

            vec![
                AuditFinding {
                    id: "compliance_001".to_string(),
                    clause_id: "cl_002".to_string(),
                    description: "适用范围符合招标投标法".to_string(),
                    severity: "low".to_string(),
                    law_ref: Some("law_002".to_string()),
                },
                AuditFinding {
                    id: "compliance_002".to_string(),
                    clause_id: "cl_042".to_string(),
                    description: "评标委员会组成符合要求".to_string(),
                    severity: "medium".to_string(),
                    law_ref: Some("law_002".to_string()),
                },
            ]
        })
    }
}

pub struct Coordinator {
    agent_registry: AgentRegistry,
    bus: AgentBus,
    session_graph: SessionGraph,
}

impl Coordinator {
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();
        registry.register("JudgeAgent".to_string(), Box::new(JudgeAgent));
        registry.register("LegalVerifyAgent".to_string(), Box::new(LegalVerifyAgent));
        registry.register("ComplianceAgent".to_string(), Box::new(ComplianceAgent));

        Self {
            agent_registry: registry,
            bus: AgentBus::new(100),
            session_graph: SessionGraph::new(),
        }
    }

    pub async fn run_coordinator(
        &mut self,
        document: Document,
        coord_definition: &CoordinatorDefinition,
    ) -> Result<AuditReport, CoordinatorError> {
        let start_time = std::time::Instant::now();

        let doc_profile = self.analyze_document(&document).await?;

        let agents = coord_definition.select_agents(&doc_profile);

        for agent_type in &agents {
            let agent_id = format!("{}_{}", agent_type, uuid::Uuid::new_v4());
            self.session_graph.add_node(agent_id, agent_type.clone());
            self.session_graph.add_edge(
                "coordinator".to_string(),
                format!("{}_{}", agent_type, agents.iter().position(|a| a == agent_type).unwrap()),
                RelationType::Parent,
            );
        }

        let findings = self.parallel_review(&document, &agents).await?;

        let validated = self.cross_validate(&findings).await?;

        let aggregated = self.aggregate_findings(&validated);

        let end_time = std::time::Instant::now();

        Ok(AuditReport {
            document_id: document.id,
            findings: aggregated,
            compliant: true,
            summary: "审核完成".to_string(),
            agent_stats: AgentStats {
                total_agents: agents.len(),
                completed_agents: self.session_graph.completed_count(),
                failed_agents: self.session_graph.failed_count(),
                total_time_ms: end_time.duration_since(start_time).as_millis() as u64,
            },
        })
    }

    async fn analyze_document(&self, document: &Document) -> Result<DocumentProfile, CoordinatorError> {
        Ok(DocumentProfile {
            doc_type: "招标文件".to_string(),
            project_type: "建筑工程".to_string(),
            complexity: 0.7,
        })
    }

    async fn parallel_review(
        &mut self,
        document: &Document,
        agents: &[String],
    ) -> Result<Vec<AuditFinding>, CoordinatorError> {
        let mut tasks = Vec::new();

        for agent_type in agents {
            let agent = self.agent_registry.get_agent(agent_type)
                .ok_or(CoordinatorError::AgentNotFound(agent_type.clone()))?;
            
            let document_clone = document.clone();
            let bus_clone = self.bus.clone();
            
            tasks.push(agent.review(&document_clone, &bus_clone));
        }

        let results = futures::future::join_all(tasks).await;
        let mut all_findings = Vec::new();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(findings) => {
                    all_findings.extend(findings);
                    let agent_id = format!("agent_{}", i);
                    self.session_graph.complete_agent(&agent_id);
                }
                Err(_) => {
                    let agent_id = format!("agent_{}", i);
                    self.session_graph.fail_agent(&agent_id);
                }
            }
        }

        Ok(all_findings)
    }

    async fn cross_validate(&self, findings: &[AuditFinding]) -> Result<Vec<AuditFinding>, CoordinatorError> {
        let mut validated = Vec::new();
        
        for finding in findings {
            let has_duplicate = validated.iter()
                .any(|v: &AuditFinding| v.clause_id == finding.clause_id && v.description == finding.description);
            
            if !has_duplicate {
                validated.push(finding.clone());
            }
        }
        
        Ok(validated)
    }

    fn aggregate_findings(&self, findings: &[AuditFinding]) -> Vec<AuditFinding> {
        let mut aggregated = findings.to_vec();
        aggregated.sort_by(|a, b| {
            let severity_order = |s: &str| match s {
                "high" => 0,
                "medium" => 1,
                "low" => 2,
                _ => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });
        aggregated
    }
}

#[derive(Debug)]
pub enum CoordinatorError {
    AgentNotFound(String),
    DocumentAnalysisFailed,
    ReviewFailed,
    AggregationFailed,
}

impl CoordinatorDefinition {
    pub fn new() -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(
            "建筑工程".to_string(),
            vec!["JudgeAgent".to_string(), "LegalVerifyAgent".to_string(), "ComplianceAgent".to_string()],
        );
        mapping.insert(
            "货物采购".to_string(),
            vec!["JudgeAgent".to_string(), "ComplianceAgent".to_string()],
        );
        mapping.insert(
            "服务招标".to_string(),
            vec!["JudgeAgent".to_string(), "LegalVerifyAgent".to_string()],
        );
        
        Self { agent_mapping: mapping }
    }

    pub fn select_agents(&self, profile: &DocumentProfile) -> Vec<String> {
        self.agent_mapping.get(&profile.project_type)
            .cloned()
            .unwrap_or_else(|| vec!["JudgeAgent".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_run() {
        let mut coordinator = Coordinator::new();
        let coord_def = CoordinatorDefinition::new();
        
        let document = Document {
            id: "doc_001".to_string(),
            content: "Test document".to_string(),
            clauses: HashMap::new(),
        };

        let report = coordinator.run_coordinator(document, &coord_def).await.unwrap();
        
        assert_eq!(report.document_id, "doc_001");
        assert!(report.findings.len() > 0);
    }

    #[tokio::test]
    async fn test_coordinator_parallel_review() {
        let mut coordinator = Coordinator::new();
        
        let document = Document {
            id: "doc_001".to_string(),
            content: "Test document".to_string(),
            clauses: HashMap::new(),
        };

        let agents = vec!["JudgeAgent".to_string(), "LegalVerifyAgent".to_string()];
        let findings = coordinator.parallel_review(&document, &agents).await.unwrap();
        
        assert!(findings.len() > 0);
    }

    #[tokio::test]
    async fn test_coordinator_agent_selection() {
        let coord_def = CoordinatorDefinition::new();
        
        let profile = DocumentProfile {
            doc_type: "招标文件".to_string(),
            project_type: "建筑工程".to_string(),
            complexity: 0.8,
        };

        let agents = coord_def.select_agents(&profile);
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_coordinator_cross_validation() {
        let coordinator = Coordinator::new();
        
        let findings = vec![
            AuditFinding {
                id: "f1".to_string(),
                clause_id: "cl_042".to_string(),
                description: "Same issue".to_string(),
                severity: "high".to_string(),
                law_ref: None,
            },
            AuditFinding {
                id: "f2".to_string(),
                clause_id: "cl_042".to_string(),
                description: "Same issue".to_string(),
                severity: "high".to_string(),
                law_ref: None,
            },
            AuditFinding {
                id: "f3".to_string(),
                clause_id: "cl_087".to_string(),
                description: "Different issue".to_string(),
                severity: "medium".to_string(),
                law_ref: None,
            },
        ];

        let validated = coordinator.cross_validate(&findings).await.unwrap();
        assert_eq!(validated.len(), 2);
    }
}
