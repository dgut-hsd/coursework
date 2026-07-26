mod probe;
mod explain;

fn main() {
    let user = "neo4j";
    let password = "b1234567";
    let api_url = "http://localhost:7474/db/neo4j/tx/commit";
    use base64::Engine;
    let auth = base64::engine::general_purpose::STANDARD
        .encode(format!("{}:{}", user, password));

    probe::probe_schema(api_url, &auth);

    let queries: [(&str, &str); 6] = [
        ("1", "MATCH (n:Law) RETURN n"),
        ("2", "MATCH (l:Law)-[:HAS_ARTICLE]->(a:Article) RETURN a"),
        (
            "3",
            "MATCH (c:Case)-[:CITED_IN]->(a:Article)<-[:HAS_ARTICLE]-(l:Law) RETURN l",
        ),
        ("4", "MATCH (r:Risk) WHERE r.severity = '高' RETURN r"),
        (
            "5a",
            "MATCH (c:Case) WHERE c.verdict = '废标' AND c.court = '财政部' RETURN c",
        ),
        (
            "5b",
            "MATCH (c:Case) WHERE c.court = '财政部' AND c.verdict = '废标' RETURN c",
        ),
    ];

    for (label, cql) in &queries {
        println!("\n{}\n", "=".repeat(60));
        println!("Query {}: {}", label, cql);
        println!("{}", "-".repeat(60));
        let plan = explain::explain_query(api_url, &auth, cql);
        explain::print_plan(&plan, 0);
    }

    println!("\n\n=== 手写分析区域 ===");
}
