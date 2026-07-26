use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SessionNode {
    pub agent_id: String,
    pub agent_type: String,
    pub status: AgentStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub children: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RelationType {
    Parent,
    Sibling,
    DependsOn,
}

#[derive(Debug, Clone)]
pub struct SessionGraph {
    nodes: HashMap<String, SessionNode>,
    edges: Vec<(String, String, RelationType)>,
}

impl SessionGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, agent_id: String, agent_type: String) {
        self.nodes.insert(
            agent_id.clone(),
            SessionNode {
                agent_id: agent_id.clone(),
                agent_type,
                status: AgentStatus::Idle,
                started_at: None,
                completed_at: None,
                children: Vec::new(),
                parent: None,
            },
        );
    }

    pub fn start_agent(&mut self, agent_id: &str) {
        if let Some(node) = self.nodes.get_mut(agent_id) {
            node.status = AgentStatus::Running;
            node.started_at = Some(Instant::now());
        }
    }

    pub fn complete_agent(&mut self, agent_id: &str) {
        if let Some(node) = self.nodes.get_mut(agent_id) {
            node.status = AgentStatus::Completed;
            node.completed_at = Some(Instant::now());
        }
    }

    pub fn fail_agent(&mut self, agent_id: &str) {
        if let Some(node) = self.nodes.get_mut(agent_id) {
            node.status = AgentStatus::Failed;
            node.completed_at = Some(Instant::now());
        }
    }

    pub fn add_edge(&mut self, from: String, to: String, relation: RelationType) {
        self.edges.push((from.clone(), to.clone(), relation.clone()));
        
        match relation {
            RelationType::Parent => {
                if let Some(parent) = self.nodes.get_mut(&from) {
                    parent.children.push(to.clone());
                }
                if let Some(child) = self.nodes.get_mut(&to) {
                    child.parent = Some(from);
                }
            }
            RelationType::DependsOn => {
                if let Some(child) = self.nodes.get_mut(&to) {
                    child.children.push(from.clone());
                }
            }
            _ => {}
        }
    }

    pub fn get_node(&self, agent_id: &str) -> Option<&SessionNode> {
        self.nodes.get(agent_id)
    }

    pub fn get_children(&self, agent_id: &str) -> Vec<&SessionNode> {
        self.nodes
            .get(agent_id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|child_id| self.nodes.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn wait_for_children(&self, agent_id: &str) -> bool {
        let children = self.get_children(agent_id);
        children.iter().all(|child| {
            child.status == AgentStatus::Completed || child.status == AgentStatus::Failed
        })
    }

    pub fn all_completed(&self) -> bool {
        self.nodes
            .values()
            .all(|node| node.status == AgentStatus::Completed)
    }

    pub fn completed_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|node| node.status == AgentStatus::Completed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|node| node.status == AgentStatus::Failed)
            .count()
    }

    pub fn running_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|node| node.status == AgentStatus::Running)
            .count()
    }

    pub fn get_agents_by_status(&self, status: AgentStatus) -> Vec<&SessionNode> {
        self.nodes
            .values()
            .filter(|node| node.status == status)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_graph_add_node() {
        let mut graph = SessionGraph::new();
        graph.add_node("agent_1".to_string(), "JudgeAgent".to_string());
        
        assert!(graph.get_node("agent_1").is_some());
    }

    #[test]
    fn test_session_graph_status_transition() {
        let mut graph = SessionGraph::new();
        graph.add_node("agent_1".to_string(), "JudgeAgent".to_string());
        
        graph.start_agent("agent_1");
        assert_eq!(graph.get_node("agent_1").unwrap().status, AgentStatus::Running);
        
        graph.complete_agent("agent_1");
        assert_eq!(graph.get_node("agent_1").unwrap().status, AgentStatus::Completed);
    }

    #[test]
    fn test_session_graph_add_edge() {
        let mut graph = SessionGraph::new();
        graph.add_node("parent".to_string(), "Coordinator".to_string());
        graph.add_node("child1".to_string(), "JudgeAgent".to_string());
        graph.add_node("child2".to_string(), "LegalAgent".to_string());
        
        graph.add_edge("parent".to_string(), "child1".to_string(), RelationType::Parent);
        graph.add_edge("parent".to_string(), "child2".to_string(), RelationType::Parent);
        
        let children = graph.get_children("parent");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_session_graph_wait_for_children() {
        let mut graph = SessionGraph::new();
        graph.add_node("parent".to_string(), "Coordinator".to_string());
        graph.add_node("child".to_string(), "JudgeAgent".to_string());
        graph.add_edge("parent".to_string(), "child".to_string(), RelationType::Parent);
        
        assert!(!graph.wait_for_children("parent"));
        
        graph.start_agent("child");
        graph.complete_agent("child");
        
        assert!(graph.wait_for_children("parent"));
    }

    #[test]
    fn test_session_graph_all_completed() {
        let mut graph = SessionGraph::new();
        graph.add_node("agent_1".to_string(), "JudgeAgent".to_string());
        graph.add_node("agent_2".to_string(), "LegalAgent".to_string());
        
        assert!(!graph.all_completed());
        
        graph.start_agent("agent_1");
        graph.complete_agent("agent_1");
        
        assert!(!graph.all_completed());
        
        graph.start_agent("agent_2");
        graph.complete_agent("agent_2");
        
        assert!(graph.all_completed());
    }

    #[test]
    fn test_session_graph_failure_tracking() {
        let mut graph = SessionGraph::new();
        graph.add_node("agent_1".to_string(), "JudgeAgent".to_string());
        
        graph.start_agent("agent_1");
        graph.fail_agent("agent_1");
        
        assert_eq!(graph.get_node("agent_1").unwrap().status, AgentStatus::Failed);
        assert_eq!(graph.failed_count(), 1);
    }
}
