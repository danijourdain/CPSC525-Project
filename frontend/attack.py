from client import Client


if __name__ == "__main__":
    
    while True:
        client = Client(
            region=0,
            addr=("0.0.0.0", 3402),
            password="fraud"
        )
        
        if client.connect():
            print("ACCESSED!")
            break
        # break
        # print(client.get_balance())