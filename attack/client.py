import socket
import struct

class Client:
    """
    The client that connects to the server.
    """
    
    def __init__(self, region: int, addr: tuple[str, int], password: str):
        """
        Creates a new client

        Args:
            region (int): the region ID to use for connection.
            addr (tuple[str, int]): the address tuple to use for connection.
            password (str): the password to use.
        """
        self.addr = addr
        self.region = region
        self.password = password
        

    def connect(self) -> bool:
        """
        Connects to the server.

        Returns:
            bool: If we managed to connect properly.
        """
        self.server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server.connect(self.addr)
        
        password = self.password.encode()
        
        self.server.sendall(b'\0' + int.to_bytes(self.region, byteorder='little', length=1) + int.to_bytes(len(password), length=4, byteorder='little') + password)
        if self.server.recv(1) == b'\1':
            return True
        else:
            return False
        
    def get_balance(self) -> int:
        """
        Looks up the balance from the server.

        Raises:
            RuntimeError: We failed to login.

        Returns:
            int: The balance as an integer.
        """
        if self.connect():
            self.server.sendall(b'\1')
            
            value = int.from_bytes(self.server.recv(4), byteorder='little', signed=True)
        
            self.server.close()
            return value
        else:
            raise RuntimeError("Could not log in.")
        
    def transact_direct(self, recipient: int, money: int):
        """
        Performs a transaction using the current stream
        state without first dropping the connection and restarting.
        
        This is key to performing the exploit.

        Args:
            recipient (int): The recipient of the money (region ID)
            money (int): The amount of money to transact.
        """
        self.server.sendall(b'\2' + int.to_bytes(self.region, length=4, byteorder='little', signed=True) + int.to_bytes(recipient, length=4, byteorder='little', signed=True) + int.to_bytes(money, length=4, byteorder='little', signed=True))
    
    def transact(self, recipient: int, money: int):
        """
        Transact the money by opening a new connection
        and then sending it.

        Args:
            recipient (int): The recipient of the money (region ID)
            money (int): The amount of money to transact.

        Raises:
            RuntimeError: We could not login properly.
        """
        if self.connect():
            self.transact_direct(recipient, money)
            self.server.close()
        else:
            raise RuntimeError("Could not log in.")
        
        
