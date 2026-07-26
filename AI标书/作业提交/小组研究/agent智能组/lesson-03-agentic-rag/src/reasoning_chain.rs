use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationStep {
    pub clause_id: String,
    pub source_quote: String,
    pub extracted_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStep {
    pub evidence_id: String,
    pub source: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStep {
    pub law_name: String,
    pub article_number: String,
    pub rule_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConclusionStep {
    pub statement: String,
    pub severity: String,
    pub law_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReasoningChain {
    pub observation: Option<ObservationStep>,
    pub evidence: Option<EvidenceStep>,
    pub rule: Option<RuleStep>,
    pub conclusion: Option<ConclusionStep>,
}

#[derive(Debug, Clone)]
pub struct ChainReport {
    pub obs_valid: bool,
    pub obs_score: f64,
    pub ev_valid: bool,
    pub rule_valid: bool,
    pub conc_valid: bool,
    pub traceable: bool,
}

pub struct ChainValidator {
    document: Document,
    tool_call_history: HashMap<String, EvidenceStep>,
    law_database: LawDatabase,
}

pub struct Document {
    clauses: HashMap<String, String>,
}

impl Document {
    pub fn new() -> Self {
        let mut clauses = HashMap::new();
        clauses.insert("cl_042".to_string(), "第四十二条 评标委员会经评审，认为所有投标都不符合招标文件要求的，可以否决所有投标。".to_string());
        clauses.insert("cl_087".to_string(), "第八十七条 招标代理机构违反本法规定，泄露应当保密的与招标投标活动有关的情况和资料的，或者与招标人、投标人串通损害国家利益、社会公共利益或者他人合法权益的，处五万元以上二十五万元以下的罚款。".to_string());
        clauses.insert("cl_001".to_string(), "第一条 为了规范招标投标活动，保护国家利益、社会公共利益和招标投标活动当事人的合法权益，提高经济效益，保证项目质量，制定本法。".to_string());
        Self { clauses }
    }

    pub fn get_clause_text(&self, clause_id: &str) -> String {
        self.clauses.get(clause_id).cloned().unwrap_or_default()
    }
}

pub struct LawDatabase {
    laws: HashMap<String, HashMap<String, String>>,
}

impl LawDatabase {
    pub fn new() -> Self {
        let mut laws = HashMap::new();
        
        let mut building_law = HashMap::new();
        building_law.insert("第12条".to_string(), "从事建筑活动的建筑施工企业、勘察单位、设计单位和工程监理单位，应当具备下列条件：（一）有符合国家规定的注册资本；（二）有与其从事的建筑活动相适应的具有法定执业资格的专业技术人员；（三）有从事相关建筑活动所应有的技术装备；（四）法律、行政法规规定的其他条件。".to_string());
        laws.insert("建筑法".to_string(), building_law);
        
        let mut tender_law = HashMap::new();
        tender_law.insert("第26条".to_string(), "投标人应当具备承担招标项目的能力；国家有关规定对投标人资格条件或者招标文件对投标人资格条件有规定的，投标人应当具备规定的资格条件。".to_string());
        tender_law.insert("第42条".to_string(), "评标委员会经评审，认为所有投标都不符合招标文件要求的，可以否决所有投标。".to_string());
        laws.insert("招标投标法".to_string(), tender_law);
        
        Self { laws }
    }

    pub fn law_article_exists(&self, law_name: &str, article_number: &str) -> bool {
        self.laws
            .get(law_name)
            .map(|articles| articles.contains_key(article_number))
            .unwrap_or(false)
    }
}

impl ChainValidator {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            tool_call_history: HashMap::new(),
            law_database: LawDatabase::new(),
        }
    }

    pub fn add_evidence(&mut self, evidence_id: String, evidence: EvidenceStep) {
        self.tool_call_history.insert(evidence_id, evidence);
    }

    pub fn validate(&self, chain: &ReasoningChain) -> ChainReport {
        let (obs_valid, obs_score) = self.validate_observation(chain);
        let ev_valid = self.validate_evidence(chain);
        let rule_valid = self.validate_rule(chain);
        let conc_valid = obs_valid && ev_valid && rule_valid;

        ChainReport {
            obs_valid,
            obs_score,
            ev_valid,
            rule_valid,
            conc_valid,
            traceable: conc_valid,
        }
    }

    fn validate_observation(&self, chain: &ReasoningChain) -> (bool, f64) {
        if let Some(obs) = &chain.observation {
            let original = self.document.get_clause_text(&obs.clause_id);
            if original.is_empty() {
                return (false, 0.0);
            }
            
            let score = fuzzy_match(&obs.source_quote, &original);
            (score > 0.85, score)
        } else {
            (false, 0.0)
        }
    }

    fn validate_evidence(&self, chain: &ReasoningChain) -> bool {
        if let Some(ev) = &chain.evidence {
            self.tool_call_history.contains_key(&ev.evidence_id)
        } else {
            false
        }
    }

    fn validate_rule(&self, chain: &ReasoningChain) -> bool {
        if let Some(rule) = &chain.rule {
            self.law_database.law_article_exists(&rule.law_name, &rule.article_number)
        } else {
            false
        }
    }
}

fn fuzzy_match(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    
    if a_chars.is_empty() || b_chars.is_empty() {
        return 0.0;
    }
    
    let mut dp = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];
    
    for i in 0..=a_chars.len() {
        dp[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        dp[0][j] = j;
    }
    
    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }
    
    let distance = dp[a_chars.len()][b_chars.len()] as f64;
    let max_len = std::cmp::max(a_chars.len(), b_chars.len()) as f64;
    
    1.0 - (distance / max_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_validation_matching() {
        let mut validator = ChainValidator::new();
        
        let chain = ReasoningChain {
            observation: Some(ObservationStep {
                clause_id: "cl_042".to_string(),
                source_quote: "评标委员会经评审，认为所有投标都不符合招标文件要求的，可以否决所有投标。".to_string(),
                extracted_text: "否决投标".to_string(),
            }),
            evidence: None,
            rule: None,
            conclusion: None,
        };

        let report = validator.validate(&chain);
        assert!(report.obs_valid);
        assert!(report.obs_score > 0.85);
    }

    #[test]
    fn test_observation_validation_not_matching() {
        let mut validator = ChainValidator::new();
        
        let chain = ReasoningChain {
            observation: Some(ObservationStep {
                clause_id: "cl_042".to_string(),
                source_quote: "完全不相关的文本".to_string(),
                extracted_text: "test".to_string(),
            }),
            evidence: None,
            rule: None,
            conclusion: None,
        };

        let report = validator.validate(&chain);
        assert!(!report.obs_valid);
    }

    #[test]
    fn test_evidence_validation() {
        let mut validator = ChainValidator::new();
        validator.add_evidence(
            "ev_001".to_string(),
            EvidenceStep {
                evidence_id: "ev_001".to_string(),
                source: "search".to_string(),
                content: "test".to_string(),
            },
        );
        
        let chain = ReasoningChain {
            observation: None,
            evidence: Some(EvidenceStep {
                evidence_id: "ev_001".to_string(),
                source: "search".to_string(),
                content: "test".to_string(),
            }),
            rule: None,
            conclusion: None,
        };

        let report = validator.validate(&chain);
        assert!(report.ev_valid);
    }

    #[test]
    fn test_rule_validation() {
        let validator = ChainValidator::new();
        
        let chain = ReasoningChain {
            observation: None,
            evidence: None,
            rule: Some(RuleStep {
                law_name: "建筑法".to_string(),
                article_number: "第12条".to_string(),
                rule_text: "从事建筑活动的企业应当具备资质。".to_string(),
            }),
            conclusion: None,
        };

        let report = validator.validate(&chain);
        assert!(report.rule_valid);
    }

    #[test]
    fn test_full_chain_validation() {
        let mut validator = ChainValidator::new();
        validator.add_evidence(
            "ev_001".to_string(),
            EvidenceStep {
                evidence_id: "ev_001".to_string(),
                source: "search".to_string(),
                content: "test".to_string(),
            },
        );
        
        let chain = ReasoningChain {
            observation: Some(ObservationStep {
                clause_id: "cl_042".to_string(),
                source_quote: "评标委员会经评审，认为所有投标都不符合招标文件要求的，可以否决所有投标。".to_string(),
                extracted_text: "否决投标".to_string(),
            }),
            evidence: Some(EvidenceStep {
                evidence_id: "ev_001".to_string(),
                source: "search".to_string(),
                content: "test".to_string(),
            }),
            rule: Some(RuleStep {
                law_name: "招标投标法".to_string(),
                article_number: "第42条".to_string(),
                rule_text: "可以否决所有投标".to_string(),
            }),
            conclusion: Some(ConclusionStep {
                statement: "条款符合法规要求".to_string(),
                severity: "low".to_string(),
                law_ref: Some("law_001".to_string()),
            }),
        };

        let report = validator.validate(&chain);
        assert!(report.traceable);
        assert!(report.conc_valid);
    }

    #[test]
    fn test_untraceable_conclusion() {
        let validator = ChainValidator::new();
        
        let chain = ReasoningChain {
            observation: None,
            evidence: None,
            rule: None,
            conclusion: Some(ConclusionStep {
                statement: "这是一个幻觉结论".to_string(),
                severity: "high".to_string(),
                law_ref: None,
            }),
        };

        let report = validator.validate(&chain);
        assert!(!report.traceable);
    }
}
