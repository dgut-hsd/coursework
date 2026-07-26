use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    FindingReported { agent: String, finding: AuditFinding },
    ReviewProgress { agent: String, progress: f64, stage: String },
    AgentError { agent: String, error: String },
    ReviewComplete { agent: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub id: String,
    pub clause_id: String,
    pub description: String,
    pub severity: String,
    pub law_ref: Option<String>,
}

#[derive(Clone)]
pub struct AgentBus {
    tx: broadcast::Sender<AgentMessage>,
}

impl AgentBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentMessage> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, msg: AgentMessage) {
        let _ = self.tx.send(msg);
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for AgentBus {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_bus_broadcast() {
        let bus = AgentBus::new(10);
        
        let mut receiver1 = bus.subscribe();
        let mut receiver2 = bus.subscribe();
        
        let msg = AgentMessage::ReviewProgress {
            agent: "test_agent".to_string(),
            progress: 50.0,
            stage: "review".to_string(),
        };
        
        bus.broadcast(msg.clone());
        
        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();
        
        assert_eq!(format!("{:?}", received1), format!("{:?}", received2));
    }

    #[tokio::test]
    async fn test_agent_bus_no_subscribers() {
        let bus = AgentBus::new(10);
        
        let msg = AgentMessage::ReviewComplete {
            agent: "test_agent".to_string(),
        };
        
        bus.broadcast(msg);
    }

    #[tokio::test]
    async fn test_agent_bus_fire_and_forget() {
        let bus = AgentBus::new(10);
        
        let msg = AgentMessage::AgentError {
            agent: "test_agent".to_string(),
            error: "test error".to_string(),
        };
        
        let _ = bus.broadcast(msg);
    }

    #[tokio::test]
    async fn test_agent_bus_multiple_subscribers() {
        let bus = AgentBus::new(10);
        
        let mut receivers = Vec::new();
        for _ in 0..5 {
            receivers.push(bus.subscribe());
        }
        
        let msg = AgentMessage::FindingReported {
            agent: "judge".to_string(),
            finding: AuditFinding {
                id: "finding_001".to_string(),
                clause_id: "cl_042".to_string(),
                description: "Test finding".to_string(),
                severity: "high".to_string(),
                law_ref: Some("law_001".to_string()),
            },
        };
        
        bus.broadcast(msg.clone());
        
        for mut receiver in receivers {
            let received = receiver.recv().await.unwrap();
            assert!(matches!(received, AgentMessage::FindingReported { .. }));
        }
    }
}
