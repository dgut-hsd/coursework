use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct Plan {
    #[serde(rename = "operatorType")]
    pub op_type_raw: String,
    pub identifiers: Vec<String>,
    #[serde(default)]
    pub children: Vec<Plan>,
    #[serde(default)]
    pub Details: Option<String>,
    #[serde(default)]
    pub EstimatedRows: Option<f64>,
    #[serde(default)]
    pub Id: Option<i64>,
}

fn post_json(api_url: &str, auth: &str, body: &Value) -> Value {
    ureq::post(api_url)
        .header("Authorization", &format!("Basic {}", auth))
        .header("Content-Type", "application/json")
        .send_json(body)
        .unwrap()
        .body_mut()
        .read_json::<Value>()
        .unwrap()
}

pub fn explain_query(api_url: &str, auth: &str, cypher: &str) -> Plan {
    let resp = post_json(
        api_url,
        auth,
        &json!({
            "statements": [{"statement": format!("EXPLAIN {}", cypher)}]
        }),
    );
    let root_val = resp["results"][0]["plan"]["root"].clone();
    serde_json::from_value(root_val).expect("failed to parse plan")
}

fn clean_op(s: &str) -> &str {
    s.strip_suffix("@neo4j").unwrap_or(s)
}

pub fn print_plan(plan: &Plan, indent: usize) {
    let pad = " ".repeat(indent);
    println!("{}{}", pad, clean_op(&plan.op_type_raw));
    if !plan.identifiers.is_empty() {
        println!("{}  identifiers: {:?}", pad, plan.identifiers);
    }
    if let Some(d) = &plan.Details {
        println!("{}  details: {}", pad, d);
    }
    if let Some(r) = plan.EstimatedRows {
        println!("{}  estimatedRows: {}", pad, r);
    }
    for child in &plan.children {
        print_plan(child, indent + 2);
    }
}
