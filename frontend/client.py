import socket
import struct

class Client:
    
    def __init__(self, region: int, addr: tuple[str, int], password: str):
        self.addr = addr
        self.region = region
        self.password = password
        

    def connect(self) -> bool:
        self.server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server.connect(self.addr)
        
        password = self.password.encode()
        
        self.server.sendall(b'\0' + int.to_bytes(self.region, byteorder='little', length=1) + int.to_bytes(len(password), length=4, byteorder='little') + password)
        
        if self.server.recv(1) == b'\1':
            return True
        else:
            return False
        
    def get_balance(self) -> int:
        if self.connect():
            self.server.sendall(b'\1')
            
            value = int.from_bytes(self.server.recv(4), byteorder='little', signed=True)
        
            self.server.close()
            return value
        else:
            raise RuntimeError("Could not log in.")
        
    def transact(self, recipient: int, money: int):
        if self.connect():
            self.server.sendall(b'\2' + int.to_bytes(self.region, length=4, byteorder='little', signed=True) + int.to_bytes(recipient, length=4, byteorder='little', signed=True) + int.to_bytes(money, length=4, byteorder='little', signed=True))
            self.server.close()
        else:
            raise RuntimeError("Could not log in.")