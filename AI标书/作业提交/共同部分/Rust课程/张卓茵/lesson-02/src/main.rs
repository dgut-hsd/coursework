use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq)]
struct Document {
    title: String,
    content: String,
}

fn search(query: &str, documents: &[Document]) -> Vec<Document> {
    //关键字查询，去重
    let keywords: HashSet<String> = query
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|word| word.chars().count() >= 2) 
        .map(str::to_lowercase)
        .collect(); 

    //对每个文档进行评分，找出包含关键字的文档
    let mut matches: Vec<(usize, Document)> = documents
        .iter() //对每个文档进行评分
        .filter_map(|document| {
            let searchable = format!("{} {}", document.title, document.content).to_lowercase();
            let score = keywords
                .iter()
                .filter(|keyword| searchable.contains(keyword.as_str())) //文本是否包含关键词
                .count();  
            (score > 0).then(|| {
                (
                    score,
                    Document {
                        title: document.title.clone(),
                        content: make_snippet(&document.content, &keywords),
                    },
                )
            })
        })
        .collect();
//按评分降序排序
    matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());//按评分降序排序
    let res: Vec<Document> = matches.into_iter().map(|(_, doc)| doc).collect();//提取文档
    res
}

//生成摘要
fn make_snippet(content: &str, keywords: &HashSet<String>) -> String {
    content
        .split("\n\n")  
        .find(|paragraph| {  
            let paragraph = paragraph.to_lowercase();
            keywords
                .iter()
                .any(|keyword| paragraph.contains(keyword.as_str()))  //：关键词集合中，只要有一个出现在段落里，就返回 true
        })
        .unwrap_or(content)  
        .chars()
        .take(80)
        .collect() 
}

fn main() {
    let documents = vec![
        Document {
            title: "政府采购法 第22条".into(),
            content: "供应商参加政府采购活动，应当具有独立承担民事责任的能力，并具有良好的商业信誉。".into(),
        },
        Document {
            title: "招标投标法 第20条".into(),
            content: "招标文件不得要求或者标明特定的生产供应者，不得含有倾向或者排斥潜在投标人的内容。".into(),
        },
        Document {
            title: "政府采购法实施条例".into(),
            content: "采购人或者采购代理机构应当根据采购项目的特点编制采购文件。\n\n资格条件不得对供应商实行差别待遇。".into(),
        },
    ];

    for result in search("政府采购 资格条件", &documents) {
        println!("{}\n  {}", result.title, result.content);
    }
}
