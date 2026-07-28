use crate::rewoo::{MockPlanner, MockExecutor, MockSolver, Planner, ToolExecutor, Solver, ExecutionResult};
use crate::reasoning_chain::{ReasoningChain, ObservationStep, EvidenceStep, RuleStep, ConclusionStep, ChainValidator, ChainReport};
use crate::knowledge_injection::{KnowledgeInjection, InjectionFormat, KnowledgeItem};

pub async fn run_demo() {
    println!("=== Lesson-03 Agentic RAG 演示 ===");
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 1：ReWOO 框架演示（规划→执行→推理）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_rewoo().await;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 2：推理链验证演示");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_reasoning_chain().await;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("任务 3：知识注入格式演示");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_knowledge_injection().await;
}

async fn demo_rewoo() {
    let user_query = "审查招标文件";
    
    println!("用户提问: {}", user_query);
    
    println!("\n--- 阶段1：规划（Planner）---");
    let planner = MockPlanner;
    let plan_result = planner.plan(user_query).await;
    if plan_result.is_err() {
        println!("规划失败: {:?}", plan_result.err().unwrap());
        return;
    }
    let plan = plan_result.unwrap();
    
    println!("生成 {} 个步骤", plan.len());
    for (i, step) in plan.iter().enumerate() {
        let deps = if let Some(deps) = &step.depends_on {
            format!("依赖步骤 {:?}", deps)
        } else {
            "无依赖".to_string()
        };
        println!("  步骤{}: 工具={}, {}", i + 1, step.tool, deps);
    }
    
    println!("\n--- 阶段2：执行（Worker）---");
    let executor = MockExecutor;
    let mut results = Vec::new();
    for step in &plan {
        let result = executor.execute(&step.tool, step.args.clone()).await;
        match result {
            Ok(output) => {
                println!("  步骤{}: {} - ✓ 成功", step.step, step.tool);
                println!("    结果: {}", output);
                results.push(ExecutionResult {
                    step: step.step,
                    tool: step.tool.clone(),
                    result: output,
                    success: true,
                });
            }
            Err(e) => {
                println!("  步骤{}: {} - ✗ 失败: {}", step.step, step.tool, e);
                results.push(ExecutionResult {
                    step: step.step,
                    tool: step.tool.clone(),
                    result: serde_json::json!({"error": format!("{}", e)}),
                    success: false,
                });
            }
        }
    }
    
    println!("\n--- 阶段3：推理（Solver）---");
    let solver = MockSolver;
    let answer = solver.solve(&plan, &results).await;
    match answer {
        Ok(a) => {
            println!("最终回答:");
            println!("  搜索结果: {}", a["search_result"]);
            println!("  条款结果: {}", a["section_result"]);
            println!("  摘要: {}", a["summary"]);
            println!("  Token消耗: {}", a["token_cost"]);
        }
        Err(e) => {
            println!("推理失败: {:?}", e);
            return;
        }
    }
    
    println!("\n✓ ReWOO 框架演示完成");
}

async fn demo_reasoning_chain() {
    println!("→ 测试1：有效推理链（完整证据链）");
    let mut validator = ChainValidator::new();
    validator.add_evidence(
        "ev_001".to_string(),
        EvidenceStep {
            evidence_id: "ev_001".to_string(),
            source: "search".to_string(),
            content: "建筑法第12条".to_string(),
        },
    );
    
    let valid_chain = ReasoningChain {
        observation: Some(ObservationStep {
            clause_id: "cl_042".to_string(),
            source_quote: "评标委员会经评审，认为所有投标都不符合招标文件要求的，可以否决所有投标。".to_string(),
            extracted_text: "否决投标".to_string(),
        }),
        evidence: Some(EvidenceStep {
            evidence_id: "ev_001".to_string(),
            source: "search".to_string(),
            content: "建筑法第12条".to_string(),
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
    
    let report = validator.validate(&valid_chain);
    print_report(&report);
    
    println!("\n→ 测试2：无效推理链（证据缺失）");
    let invalid_chain = ReasoningChain {
        observation: None,
        evidence: None,
        rule: None,
        conclusion: Some(ConclusionStep {
            statement: "这是一个幻觉结论".to_string(),
            severity: "high".to_string(),
            law_ref: None,
        }),
    };
    
    let report = validator.validate(&invalid_chain);
    print_report(&report);
    
    println!("\n→ 测试3：无效推理链（观察不匹配）");
    let mismatched_chain = ReasoningChain {
        observation: Some(ObservationStep {
            clause_id: "cl_042".to_string(),
            source_quote: "完全不相关的文本".to_string(),
            extracted_text: "test".to_string(),
        }),
        evidence: None,
        rule: None,
        conclusion: None,
    };
    
    let report = validator.validate(&mismatched_chain);
    print_report(&report);
    
    println!("\n✓ 推理链验证演示完成");
}

fn print_report(report: &ChainReport) {
    println!("  观察有效: {} (分数: {:.2})", report.obs_valid, report.obs_score);
    println!("  证据有效: {}", report.ev_valid);
    println!("  规则有效: {}", report.rule_valid);
    println!("  结论有效: {}", report.conc_valid);
    println!("  可追溯: {}", report.traceable);
}

async fn demo_knowledge_injection() {
    let items = vec![
        KnowledgeItem {
            law_id: "law_001".to_string(),
            law_name: "建筑法".to_string(),
            article: "第12条".to_string(),
            text: "从事建筑活动的建筑施工企业、勘察单位、设计单位和工程监理单位，应当具备相应资质。".to_string(),
            relevance: 0.95,
        },
        KnowledgeItem {
            law_id: "law_002".to_string(),
            law_name: "招标投标法".to_string(),
            article: "第26条".to_string(),
            text: "投标人应当具备承担招标项目的能力，具备规定的资格条件。".to_string(),
            relevance: 0.88,
        },
    ];
    
    println!("格式A：纯文本");
    let plain_text = KnowledgeInjection::inject(&items, InjectionFormat::PlainText);
    println!("{}", plain_text);
    
    println!("格式B：结构化JSON");
    let json_text = KnowledgeInjection::inject(&items, InjectionFormat::StructuredJson);
    println!("{}", json_text);
    
    println!("格式C：引用标注");
    let citation_text = KnowledgeInjection::inject(&items, InjectionFormat::CitationMarkup);
    println!("{}", citation_text);
    
    let plain_tokens = KnowledgeInjection::calculate_overhead(&items, InjectionFormat::PlainText);
    let json_tokens = KnowledgeInjection::calculate_overhead(&items, InjectionFormat::StructuredJson);
    let citation_tokens = KnowledgeInjection::calculate_overhead(&items, InjectionFormat::CitationMarkup);
    
    println!("Token消耗对比:");
    println!("  纯文本: {} tokens", plain_tokens);
    println!("  JSON: {} tokens", json_tokens);
    println!("  引用标注: {} tokens", citation_tokens);
    
    println!("\n✓ 知识注入格式演示完成");
}
