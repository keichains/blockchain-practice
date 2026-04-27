use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use serde::Serialize;
use std::fs::File;
use std::io::Write;

type Hash = [u8; 32];

fn to_hex(hash: &Hash) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Clone, Debug)]
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    value: Hash,      // hash của node
    content: String,  // tên file hoặc mô tả
    is_copied: bool,  // node padding khi số leaf lẻ
}

impl Node {
    fn new(
        left: Option<Box<Node>>,
        right: Option<Box<Node>>,
        value: Hash,
        content: String,
        is_copied: bool,
    ) -> Self {
        Self {
            left,
            right,
            value,
            content,
            is_copied,
        }
    }

    fn hash_bytes(data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&[0x00]); // Đánh dấu đây là LEAF NODE
        hasher.update(data);
        hasher.finalize().into()
}

    fn hash_pair(left: &Hash, right: &Hash) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&[0x01]); // Đánh dấu đây là INTERMEDIATE NODE
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
}

    fn copy_node(&self) -> Node {
        Node {
            left: self.left.clone(),
            right: self.right.clone(),
            value: self.value,
            content: self.content.clone(),
            is_copied: true,
        }
    }
}

#[derive(Debug)]
struct MerkleTree {
    root: Node,
}

#[derive(Clone, Debug)]
struct ProofStep {
    sibling_hash: Hash,
    // true: sibling ở bên trái current hash
    // false: sibling ở bên phải current hash
    sibling_is_left: bool,
}

#[derive(Debug, Clone)]
struct FileLeaf {
    file_name: String,
    file_hash: Hash,
}

impl MerkleTree {
    fn from_file_leaves(file_leaves: Vec<FileLeaf>) -> Self {
        let root = Self::build_tree_from_file_leaves(file_leaves);
        Self { root }
    }

    fn build_tree_from_file_leaves(file_leaves: Vec<FileLeaf>) -> Node {
        if file_leaves.is_empty() {
            panic!("Không có file nào trong thư mục sample_docs");
        }

        let mut leaves: Vec<Node> = file_leaves
            .into_iter()
            .map(|f| Node::new(None, None, f.file_hash, f.file_name, false))
            .collect();

        if leaves.len() % 2 == 1 && leaves.len() > 1 {
            let last = leaves.last().unwrap().copy_node();
            leaves.push(last);
        }

        Self::build_tree_rec(leaves)
    }

    fn build_tree_rec(mut nodes: Vec<Node>) -> Node {
        if nodes.len() == 1 {
            return nodes.remove(0);
        }

        if nodes.len() % 2 == 1 {
            let last = nodes.last().unwrap().copy_node();
            nodes.push(last);
        }

        if nodes.len() == 2 {
            let mut iter = nodes.into_iter();
            let left = iter.next().unwrap();
            let right = iter.next().unwrap();

            let value = Node::hash_pair(&left.value, &right.value);
            let content = format!("{} + {}", left.content, right.content);

            return Node::new(Some(Box::new(left)), Some(Box::new(right)), value, content, false);
        }

        let half = nodes.len() / 2;
        let right_nodes = nodes.split_off(half);
        let left = Self::build_tree_rec(nodes);
        let right = Self::build_tree_rec(right_nodes);

        let value = Node::hash_pair(&left.value, &right.value);
        let content = format!("{} + {}", left.content, right.content);

        Node::new(Some(Box::new(left)), Some(Box::new(right)), value, content, false)
    }

    fn get_root_hash_hex(&self) -> String {
        to_hex(&self.root.value)
    }

    fn get_root_raw(&self) -> Hash {
        self.root.value
    }

    fn print_tree(&self) {
        Self::print_tree_rec(&self.root);
    }

    fn print_tree_rec(node: &Node) {
        if let Some(left) = &node.left {
            println!("Left : {}", to_hex(&left.value));
            if let Some(right) = &node.right {
                println!("Right: {}", to_hex(&right.value));
            }
        } else {
            println!("Leaf");
        }

        if node.is_copied {
            println!("(Padding node)");
        }

        println!("Value  : {}", to_hex(&node.value));
        println!("Content: {}", node.content);
        println!();

        if let Some(left) = &node.left {
            Self::print_tree_rec(left);
        }
        if let Some(right) = &node.right {
            Self::print_tree_rec(right);
        }
    }

    fn generate_proof(&self, target_file_name: &str) -> Option<Vec<ProofStep>> {
        let mut proof = Vec::new();
        let found = Self::generate_proof_rec(&self.root, target_file_name, &mut proof);

        if found {
            Some(proof)
        } else {
            None
        }
    }

    fn generate_proof_rec(node: &Node, target_file_name: &str, proof: &mut Vec<ProofStep>) -> bool {
        // leaf
        if node.left.is_none() && node.right.is_none() {
            return node.content == target_file_name;
        }

        let left = node.left.as_ref().unwrap();
        let right = node.right.as_ref().unwrap();

        if Self::generate_proof_rec(left, target_file_name, proof) {
            proof.push(ProofStep {
                sibling_hash: right.value,
                sibling_is_left: false,
            });
            return true;
        }

        if Self::generate_proof_rec(right, target_file_name, proof) {
            proof.push(ProofStep {
                sibling_hash: left.value,
                sibling_is_left: true,
            });
            return true;
        }

        false
    }

    fn verify_proof_from_hash(leaf_hash: Hash, proof: &[ProofStep], expected_root: &Hash) -> bool {
        let mut current_hash = leaf_hash;

        for step in proof {
            current_hash = if step.sibling_is_left {
                Node::hash_pair(&step.sibling_hash, &current_hash)
            } else {
                Node::hash_pair(&current_hash, &step.sibling_hash)
            };
        }

        &current_hash == expected_root
    }
}

fn hash_file(path: &Path) -> Result<Hash, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    Ok(Node::hash_bytes(&data))
}

fn load_files_from_dir(dir: &str) -> Result<Vec<FileLeaf>, Box<dyn std::error::Error>> {
    let mut file_paths: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            file_paths.push(path);
        }
    }

    // Sắp xếp để thứ tự cố định, tránh mỗi lần chạy root khác nhau
    file_paths.sort();

    let mut leaves = Vec::new();

    for path in file_paths {
        let file_name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let file_hash = hash_file(&path)?;

        leaves.push(FileLeaf { file_name, file_hash });
    }

    Ok(leaves)
}

// Xuất file ra json
#[derive(Serialize)]
struct FileRecord {
    file_name: String,
    file_hash: String,
}

#[derive(Serialize)]
struct BatchRecord {
    batch_id: String,
    timestamp: String,
    merkle_root: String,
    files: Vec<FileRecord>,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = "../sample_docs";
    let file_leaves = load_files_from_dir(dir)?;
    let mtree = MerkleTree::from_file_leaves(file_leaves.clone());
    let batch_record = BatchRecord {
        batch_id: "batch_001".to_string(),
        timestamp: format!("{:?}", std::time::SystemTime::now()),
        merkle_root: mtree.get_root_hash_hex(),
        files: file_leaves
            .iter()
            .map(|leaf| FileRecord {
                file_name: leaf.file_name.clone(),
                file_hash: to_hex(&leaf.file_hash),
            })
            .collect(),
        };
    if file_leaves.is_empty() {
        println!("Thư mục '{}' không có file nào.", dir);
        return Ok(());
    }
    // Tạo thư mục output nếu chưa có
    fs::create_dir_all("../output")?;

    // Chuyển sang JSON đẹp
    let json_data = serde_json::to_string_pretty(&batch_record)?;

    // Ghi ra file
    let mut file = File::create("../output/batch_record.json")?;
    file.write_all(json_data.as_bytes())?;

    println!("Đã xuất file JSON: output/batch_record.json");
    println!("=== INPUT FILES ===");
    for leaf in &file_leaves {
        println!("{} -> {}", leaf.file_name, to_hex(&leaf.file_hash));
    }
    println!();

    

    println!("=== ROOT HASH ===");
    println!("{}\n", mtree.get_root_hash_hex());

    println!("=== TREE ===");
    mtree.print_tree();

    // Chọn file cần generate proof
    let target_file = "Chuong 9.pdf";
    println!("=== GENERATE PROOF FOR '{}' ===", target_file);

    match mtree.generate_proof(target_file) {
        Some(proof) => {
            for (i, step) in proof.iter().enumerate() {
                println!(
                    "Step {}: sibling_hash = {}, sibling_is_left = {}",
                    i + 1,
                    to_hex(&step.sibling_hash),
                    step.sibling_is_left
                );
            }

            // Hash lại file target để verify
            let target_path = Path::new(dir).join(target_file);
            let target_hash = hash_file(&target_path)?;

            let is_valid =
                MerkleTree::verify_proof_from_hash(target_hash, &proof, &mtree.get_root_raw());

            println!("\nVerify result for '{}': {}", target_file, is_valid);

            // Test case sai: hash từ nội dung giả
            let fake_hash = Node::hash_bytes(b"fake file content");
            let fake_result =
                MerkleTree::verify_proof_from_hash(fake_hash, &proof, &mtree.get_root_raw());

            println!("Verify result for fake content: {}", fake_result);
        }
        None => {
            println!("Không tìm thấy file '{}' trong Merkle Tree", target_file);
        }
    }

    Ok(())
}