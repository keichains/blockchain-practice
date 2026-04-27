# blockchain-practice
Cách chạy thử:
  Tải Rust 
  Cho một vài file .docs .pdf,... vào folder sample_docs
  cargo run file main.rs trong phần merkle_tree
  chạy file add_batch_to_blockchain.py Các file được gom thành một batch Hệ thống dùng Merkle Tree để:
    
    hash từng file
    tạo Merkle Root đại diện cho toàn bộ batch Thông tin chi tiết của batch được lưu ở folder output dưới dạng batch_record.json
    Blockchain không chuyển nguyên JSON thành block theo kiểu lưu toàn bộ chi tiết, mà lấy ra các thông tin quan trọng từ JSON như: batch_id,merkle_root, manifest_hash, document_count
    Sau đó ghi các thông tin này vào một block
    Chuỗi block liên kết bằng previous_hash, giúp phát hiện nếu dữ liệu block bị sửa đổi
