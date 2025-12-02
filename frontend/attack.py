from client import Client


if __name__ == "__main__":
    print("Running the attack script...")
    while True:
        client = Client(
            region=0,
            addr=("0.0.0.0", 3402),
            password="fraud"
        )
        
        if client.connect():
            client.transact_direct(1, 38)
            print("ACCESSED!")
            break