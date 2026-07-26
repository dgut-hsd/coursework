use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const NODE_RECORD_SIZE: usize = 15;
const REL_RECORD_SIZE: usize = 34;

// 与任务1 parse_node_record 返回值匹配的结构体
#[derive(Debug)]
struct NodeRecord {
    in_use: bool,
    is_dense: bool,
    first_rel_id: i32,
    first_prop_id: i32,
}

// 与任务1 parse_rel_record 返回值匹配的结构体
#[derive(Debug)]
struct RelRecord {
    first_node_id: i32,
    second_node_id: i32,
    rel_type: i32,
    first_prev_rel_id: i32,
    first_next_rel_id: i32,
    second_prev_rel_id: i32,
    second_next_rel_id: i32,
    first_prop_id: i32,
}

// ---------- 任务1提供的函数（一字不改） ----------
fn parse_node_record(bytes: &[u8; 15]) -> NodeRecord {
    NodeRecord {
        in_use: bytes[0] & 1 == 1,
        is_dense: bytes[0] & 2 == 2,
        first_rel_id: i32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
        first_prop_id: i32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
    }
}

fn parse_rel_record(bytes: &[u8; 34]) -> RelRecord {
    RelRecord {
        first_node_id: i32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
        second_node_id: i32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        rel_type: i32::from_be_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]),
        first_prev_rel_id: i32::from_be_bytes([bytes[13], bytes[14], bytes[15], bytes[16]]),
        first_next_rel_id: i32::from_be_bytes([bytes[17], bytes[18], bytes[19], bytes[20]]),
        second_prev_rel_id: i32::from_be_bytes([bytes[21], bytes[22], bytes[23], bytes[24]]),
        second_next_rel_id: i32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]),
        first_prop_id: i32::from_be_bytes([bytes[29], bytes[30], bytes[31], bytes[32]]),
    }
}

// ---------- 辅助函数：读取指定偏移和大小的字节 ----------
fn read_bytes(file: &mut File, offset: u64, size: usize) -> Vec<u8> {
    file.seek(SeekFrom::Start(offset)).expect("seek failed");
    let mut buf = vec![0u8; size];
    file.read_exact(&mut buf).expect("read failed");
    buf
}

fn main() {
    // 请根据你的 Neo4j 数据目录修改
    let data_dir = Path::new("D:\\neo4j\\data\\databases\\neo4j");
    let node_path = data_dir.join("neostore.nodestore.db");
    let rel_path = data_dir.join("neostore.relationshipstore.db");

    if !node_path.exists() {
        eprintln!("找不到节点文件: {:?}", node_path);
        eprintln!("请修改 data_dir 为你的 Neo4j 数据目录");
        return;
    }

    let mut node_file = File::open(&node_path).expect("无法打开节点文件");
    let mut rel_file = File::open(&rel_path).expect("无法打开关系文件");

    let node_len = node_file.metadata().unwrap().len();
    let rel_len = rel_file.metadata().unwrap().len();

    println!("============================================================");
    println!("前 10 个节点记录 (Node Records)");
    println!("============================================================");

    let max_nodes = std::cmp::min(10, node_len as usize / NODE_RECORD_SIZE);
    for i in 0..max_nodes {
        let offset = (i as u64) * NODE_RECORD_SIZE as u64;
        let buf: [u8; 15] = read_bytes(&mut node_file, offset, 15)
            .try_into()
            .expect("节点记录长度错误");
        let rec = parse_node_record(&buf);
        if rec.in_use {
            println!(
                "  Node ID={:5} | in_use={} dense={} | firstRelId={:5} | firstPropId={:5}",
                i, rec.in_use, rec.is_dense, rec.first_rel_id, rec.first_prop_id
            );
        } else {
            println!("  Node ID={:5} | [deleted]", i);
        }
    }

    println!();
    println!("============================================================");
    println!("前 10 条关系记录 (Relationship Records)");
    println!("============================================================");

    let max_rels = std::cmp::min(10, rel_len as usize / REL_RECORD_SIZE);
    for i in 0..max_rels {
        let offset = (i as u64) * REL_RECORD_SIZE as u64;
        let buf: [u8; 34] = read_bytes(&mut rel_file, offset, 34)
            .try_into()
            .expect("关系记录长度错误");
        let rec = parse_rel_record(&buf);
        // 注意：任务1的 parse_rel_record 没有返回 in_use，这里我们假设所有记录都在用
        println!(
            "  Rel ID={:5} | firstNode={:5} secondNode={:5} | type={:5} | firstNext={:5} secondNext={:5}",
            i, rec.first_node_id, rec.second_node_id, rec.rel_type,
            rec.first_next_rel_id, rec.second_next_rel_id
        );
    }

    // ---------- 验证 Node(id=0) 的 firstRelId ----------
    println!("\n============================================================");
    println!("验证：Node id=0 的 firstRelId");
    println!("============================================================");

    if node_len >= NODE_RECORD_SIZE as u64 {
        let buf: [u8; 15] = read_bytes(&mut node_file, 0, 15)
            .try_into()
            .expect("读取节点0失败");
        let node0 = parse_node_record(&buf);
        if node0.in_use {
            let first_rel = node0.first_rel_id;
            println!("  Node(0).firstRelId = {}", first_rel);
            if first_rel >= 0 {
                let rel_offset = (first_rel as u64) * REL_RECORD_SIZE as u64;
                if rel_offset + REL_RECORD_SIZE as u64 <= rel_len {
                    let buf: [u8; 34] = read_bytes(&mut rel_file, rel_offset, 34)
                        .try_into()
                        .expect("读取关系记录失败");
                    let rel_rec = parse_rel_record(&buf);
                    println!(
                        "  对应关系记录: Rel(id={}) firstNode={} secondNode={}",
                        first_rel, rel_rec.first_node_id, rel_rec.second_node_id
                    );
                    if rel_rec.first_node_id == 0 || rel_rec.second_node_id == 0 {
                        println!("  ✓ 验证通过：关系中包含节点 0");
                    } else {
                        println!("  ✗ 验证失败：关系中未包含节点 0");
                    }
                } else {
                    println!("  ✗ 偏移超出关系文件大小");
                }
            } else {
                println!("  Node(0) 没有关联关系（firstRelId = -1）");
            }
        } else {
            println!("  Node(0) 未使用");
        }
    } else {
        println!("  节点文件太小");
    }
}