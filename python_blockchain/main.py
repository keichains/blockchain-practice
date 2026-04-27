import hashlib
import datetime
import json

class Block:
    ##Attributes
    def __init__(self, index: int, data: dict, previous_hash: str):
        self.index = int(index)
        self.timestamp = str(datetime.datetime.now())
        self.data = data # data str -> data dict
        self.previous_hash = str(previous_hash)
        self.hash = self.calculate_hash()
    ##calculate_hash
    def calculate_hash(self):
        block_string = json.dumps({
            "index": self.index,
            "timestamp": self.timestamp,
            "data": self.data,
            "previous_hash": self.previous_hash
        }, sort_keys=True, ensure_ascii=False)
        return hashlib.sha256(block_string.encode()).hexdigest()
    
class Blockchain:
    ##Khởi tạo
    def __init__(self):
        self.chain = []
        self.create_genesis_block()
    ##Tạo block đầu tiên
    def create_genesis_block(self):
        genesis_block = Block(0, {"message": "Genesis Block"}, "0")
        self.chain.append(genesis_block)
    ##Phương thức get_last_block(self)
    def get_last_block(self):
        return self.chain[-1]
    #Phương thức add_block(self, data)
    def add_block(self, data: str):
        last_block = self.get_last_block()
        new_block = Block(index=last_block.index + 1, 
                          data = data, 
                          previous_hash=last_block.hash)
        self.chain.append(new_block)
    ##Phương thức is_chain_valid(self)
    def is_valid(self):
        for i in range(1, len(self.chain)):
            current = self.chain[i]
            pre_block = self.chain[i-1]
            if current.hash != current.calculate_hash(): ##check hash đang lưu có khớp với hash được tính lại không
                return False
            if current.previous_hash != pre_block.hash: ##check previous hash có khớp với hash của block trước không
                return False
        return True ##chạy xong tất cả block mà không lỗi trả về True
    def print_chain(self):
        for block in self.chain:
            print(f"===== Block {block.index} =====")
            print(f"Timestamp     : {block.timestamp}")
            print(f"Data          : {json.dumps(block.data, ensure_ascii=False, indent=2)}")
            print(f"Hash          : {block.hash}")
            print(f"Previous Hash : {block.previous_hash}\n")