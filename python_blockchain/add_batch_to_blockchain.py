import hashlib
import json
from pathlib import Path
from main import Blockchain

def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()

def load_batch_record(json_path: str) -> dict:
    with open(json_path, "r", encoding="utf-8") as f:
        return json.load(f)

def calculate_manifest_hash(batch_record: dict) -> str:
    normalized = json.dumps(batch_record, sort_keys=True, ensure_ascii=False)
    return sha256_text(normalized)

if __name__ == "__main__":
    json_path = "output/batch_record.json"

    batch_record = load_batch_record(json_path)
    manifest_hash = calculate_manifest_hash(batch_record)

    blockchain = Blockchain()

    block_data = {
        "batch_id": batch_record["batch_id"],
        "merkle_root": batch_record["merkle_root"],
        "manifest_hash": manifest_hash,
        "document_count": len(batch_record["files"]),
        "note": "Document batch registration"
    }

    blockchain.add_block(block_data)

    print("=== BLOCKCHAIN ===")
    blockchain.print_chain()

    print("Blockchain valid:", blockchain.is_valid())