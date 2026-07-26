use serde_json::{json, Value};

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

pub fn probe_schema(api_url: &str, auth: &str) {
    println!("\n{}\n", "=".repeat(60));
    println!("Schema Probe");
    println!("{}\n", "=".repeat(60));

    let resp = post_json(
        api_url,
        auth,
        &json!({
            "statements": [
                {"statement": "CALL db.labels()"},
                {"statement": "CALL db.relationshipTypes()"},
                {"statement": "CALL db.indexes()"},
            ]
        }),
    );
    let results = resp["results"].as_array().unwrap();

    if let Some(arr) = results[0]["data"].as_array() {
        print!("Labels [{}]: ", arr.len());
        for d in arr {
            print!("{}  ", d["row"][0].as_str().unwrap());
        }
        println!();
    }

    if let Some(arr) = results[1]["data"].as_array() {
        print!("Rels [{}]: ", arr.len());
        for d in arr {
            print!("{}  ", d["row"][0].as_str().unwrap());
        }
        println!();
    }

    if !resp["errors"].as_array().map(|e| e.is_empty()).unwrap_or(true) {
        println!("Errors: {:?}", resp["errors"]);
    }
    println!("results len = {}", results.len());
    if let Some(r3) = results.get(2) {
        if let Some(arr) = r3["data"].as_array() {
            let cols: Vec<&str> = r3["columns"]
                .as_array()
                .map(|c| c.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            println!("Indexes [{}]: columns={:?}", arr.len(), cols);
            for d in arr {
                let row = d["row"].as_array().cloned().unwrap_or_default();
                let name = row.get(0).and_then(|v| v.as_str()).unwrap_or("?");
                let typ = row.get(1).and_then(|v| v.as_str()).unwrap_or("?");
                let entity = row.get(2).and_then(|v| v.as_str()).unwrap_or("?");
                let props = row
                    .get(3)
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                println!("  {}  ON {} ({})  [{}]", name, entity, props, typ);
            }
        }
    } else {
        println!("Indexes: (no result — statement may have errored)");
    }

    for label in &["Law", "Article", "Case", "Risk", "Prohibition"] {
        let resp = post_json(
            api_url,
            auth,
            &json!({
                "statements": [{
                    "statement": format!(
                        "MATCH (n:{}) RETURN count(*) AS cnt, keys(n) AS ks LIMIT 1", label
                    )
                }]
            }),
        );
        if let Some(d) = resp["results"][0]["data"]
            .as_array()
            .and_then(|a| a.first())
        {
            let r = &d["row"];
            let cnt = r[0].as_i64().unwrap_or(0);
            let props: Vec<String> = r[1]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            println!("  {} ({} nodes): properties = {:?}", label, cnt, props);
        }
    }
}
