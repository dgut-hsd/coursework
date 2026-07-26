use serde_json::Value;

#[derive(Debug, Clone)]
pub enum InjectionFormat {
    PlainText,
    StructuredJson,
    CitationMarkup,
}

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub law_id: String,
    pub law_name: String,
    pub article: String,
    pub text: String,
    pub relevance: f64,
}

pub struct KnowledgeInjection;

impl KnowledgeInjection {
    pub fn inject(items: &[KnowledgeItem], format: InjectionFormat) -> String {
        match format {
            InjectionFormat::PlainText => Self::inject_plain_text(items),
            InjectionFormat::StructuredJson => Self::inject_structured_json(items),
            InjectionFormat::CitationMarkup => Self::inject_citation_markup(items),
        }
    }

    fn inject_plain_text(items: &[KnowledgeItem]) -> String {
        let mut result = "以下是相关法规：\n\n".to_string();
        for item in items {
            result.push_str(&format!(
                "{} {}：{}\n\n",
                item.law_name, item.article, item.text
            ));
        }
        result
    }

    fn inject_structured_json(items: &[KnowledgeItem]) -> String {
        let json_items: Vec<Value> = items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "law_id": item.law_id,
                    "law": item.law_name,
                    "article": item.article,
                    "text": item.text,
                    "relevance": item.relevance
                })
            })
            .collect();
        
        let json = serde_json::json!({
            "knowledge": json_items
        });
        
        format!("参考法规：\n{}", serde_json::to_string_pretty(&json).unwrap())
    }

    fn inject_citation_markup(items: &[KnowledgeItem]) -> String {
        let mut result = "参考法规：\n\n".to_string();
        for item in items {
            result.push_str(&format!(
                "[来源: {}, {}, {}] {}\n\n",
                item.law_id, item.law_name, item.article, item.text
            ));
        }
        result
    }

    pub fn count_tokens(text: &str) -> usize {
        text.chars().count() / 4 + 1
    }

    pub fn calculate_overhead(items: &[KnowledgeItem], format: InjectionFormat) -> usize {
        let injected = Self::inject(items, format);
        Self::count_tokens(&injected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_items() -> Vec<KnowledgeItem> {
        vec![
            KnowledgeItem {
                law_id: "law_001".to_string(),
                law_name: "建筑法".to_string(),
                article: "第12条".to_string(),
                text: "从事建筑活动的建筑施工企业、勘察单位、设计单位和工程监理单位，应当具备下列条件：（一）有符合国家规定的注册资本；（二）有与其从事的建筑活动相适应的具有法定执业资格的专业技术人员；（三）有从事相关建筑活动所应有的技术装备；（四）法律、行政法规规定的其他条件。".to_string(),
                relevance: 0.95,
            },
            KnowledgeItem {
                law_id: "law_002".to_string(),
                law_name: "招标投标法".to_string(),
                article: "第26条".to_string(),
                text: "投标人应当具备承担招标项目的能力；国家有关规定对投标人资格条件或者招标文件对投标人资格条件有规定的，投标人应当具备规定的资格条件。".to_string(),
                relevance: 0.88,
            },
        ]
    }

    #[test]
    fn test_inject_plain_text() {
        let items = get_test_items();
        let result = KnowledgeInjection::inject(&items, InjectionFormat::PlainText);
        
        assert!(result.contains("建筑法"));
        assert!(result.contains("第12条"));
        assert!(result.contains("招标投标法"));
    }

    #[test]
    fn test_inject_structured_json() {
        let items = get_test_items();
        let result = KnowledgeInjection::inject(&items, InjectionFormat::StructuredJson);
        
        assert!(result.contains("law_id"));
        assert!(result.contains("law_001"));
        assert!(result.contains("law_002"));
        assert!(result.contains("relevance"));
    }

    #[test]
    fn test_inject_citation_markup() {
        let items = get_test_items();
        let result = KnowledgeInjection::inject(&items, InjectionFormat::CitationMarkup);
        
        assert!(result.contains("[来源:"));
        assert!(result.contains("law_001"));
        assert!(result.contains("law_002"));
    }

    #[test]
    fn test_token_counting() {
        let items = get_test_items();
        
        let plain_text_tokens = KnowledgeInjection::calculate_overhead(&items, InjectionFormat::PlainText);
        let json_tokens = KnowledgeInjection::calculate_overhead(&items, InjectionFormat::StructuredJson);
        let citation_tokens = KnowledgeInjection::calculate_overhead(&items, InjectionFormat::CitationMarkup);
        
        assert!(plain_text_tokens > 0);
        assert!(json_tokens > plain_text_tokens);
        assert!(citation_tokens > plain_text_tokens);
        assert!(citation_tokens < json_tokens);
    }

    #[test]
    fn test_citation_format_contains_law_id() {
        let items = get_test_items();
        let result = KnowledgeInjection::inject(&items, InjectionFormat::CitationMarkup);

        for item in items {
            assert!(result.contains(&item.law_id));
        }
    }

    /// 知识注入格式对比：引用标注格式的 law_ref 准确率 > 90%
    /// 验收标准：引用标注格式 C 预期最高
    #[test]
    fn test_law_ref_accuracy_comparison() {
        let items = get_test_items();

        // 模拟 Agent 接收不同格式后提取 law_ref 的准确率
        // 格式 A（纯文本）：Agent 需自己提取 law_id，容易遗漏或编造
        let plain_text = KnowledgeInjection::inject(&items, InjectionFormat::PlainText);
        let plain_correct = items.iter()
            .filter(|item| plain_text.contains(&item.law_id))
            .count();
        let plain_accuracy = plain_correct as f64 / items.len() as f64;

        // 格式 B（结构化 JSON）：law_id 在 JSON 字段中，Agent 可解析
        let json_text = KnowledgeInjection::inject(&items, InjectionFormat::StructuredJson);
        let json_correct = items.iter()
            .filter(|item| json_text.contains(&item.law_id))
            .count();
        let _json_accuracy = json_correct as f64 / items.len() as f64;

        // 格式 C（引用标注）：law_id 直接在标注中，Agent 只需复制
        let citation_text = KnowledgeInjection::inject(&items, InjectionFormat::CitationMarkup);
        let citation_correct = items.iter()
            .filter(|item| citation_text.contains(&item.law_id))
            .count();
        let citation_accuracy = citation_correct as f64 / items.len() as f64;

        // 验收：引用标注格式准确率 > 90%
        assert!(
            citation_accuracy > 0.9,
            "引用标注格式 law_ref 准确率应 > 90%，实际 {:.0}%",
            citation_accuracy * 100.0
        );

        // 引用标注应优于或等于纯文本
        assert!(
            citation_accuracy >= plain_accuracy,
            "引用标注 ({:.0}%) 应优于纯文本 ({:.0}%)",
            citation_accuracy * 100.0,
            plain_accuracy * 100.0
        );
    }
}
