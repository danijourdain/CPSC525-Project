from client import Client


if __name__ == "__main__":
    print("Running the attack script...")
    while True:
        # Start a client at Calgary (id = 0)
        # connect to the address @ port 3402
        # try connecting with a password that is most
        # definitely incorrect.
        client = Client(
            region=0,
            addr=("0.0.0.0", 3402),
            password="fraud"
        )
        
        # Keep connecting until eventually the bug
        # happens and we transact 38 to New York.
        if client.connect():
            client.transact_direct(1, 38)
            print("ACCESSED!")
            break