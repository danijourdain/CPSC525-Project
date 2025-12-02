import curses
from client import Client
import time


CLIENT: Client = None
BALANCE: int = -1
INCR: int = -1
SHOULD_KILL: bool = False

def try_make_client(password):
    client = Client(region=0, addr=("0.0.0.0", 3402), password=password)
    if client.connect():
        return client
    else:
        return None

import threading

def client_watch():
    global BALANCE, CLIENT
    while True:
        
        if SHOULD_KILL:
            break
        
        if CLIENT is not None:
            # print("HI")
            BALANCE = CLIENT.get_balance()
            # BALANCE += 1
        # BALANCE += 1
        time.sleep(0.1)
        

def main(stdscr):
    global BALANCE, CLIENT, SHOULD_KILL
    curses.use_default_colors()
    curses.curs_set(0)          # hide cursor
    stdscr.timeout(200)
    curses.init_pair(1, curses.COLOR_GREEN  , curses.A_NORMAL)
    # stdscr.nodelay(False)       # blocking getch
    stdscr.keypad(True)         # enable arrows
    
    threading.Thread(target=client_watch).start()

    menu = ["View balances", "Make transfer", "Quit"]
    selected = 0

    client = try_make_client(password="bluecircle123")
    logged_in = True
    logging_in = False
    
    CLIENT = client

    while True:
        stdscr.clear()
        stdscr.addstr(0, 0, "Trading Desk " + str(logged_in) + " " + str(logging_in))
        
        
        if not logged_in:
            stdscr.addstr(2, 2, "LOGIN")    
        else:
            stdscr.addstr(2, 0, "Region:\t\t" + str(client.region))
            stdscr.addstr(3, 0, f"Balance:\t${BALANCE:,}")
            stdscr.addstr(4, 0, f'Status:\t\tCONNECTED')

        for i, item in enumerate(menu):
            if i == selected:
                stdscr.attron(curses.COLOR_GREEN)
                stdscr.addstr(6 + i, 2, '>> ' + item, curses.color_pair(1))
                stdscr.attroff(curses.COLOR_GREEN)
            else:
                stdscr.addstr(6 + i, 2, '> ' + item)
            # if i == selected:
            #     stdscr.attron(curses.A_REVERSE)
                
            # stdscr.addstr(5 + i, 2, '> ' + item)
            # if i == selected:
            #     stdscr.attroff(curses.A_REVERSE)

        stdscr.refresh()

        key = stdscr.getch()
     
     
        if key == -1:
            continue
 
        if not logged_in and not logging_in:
            if key in (curses.KEY_ENTER, 10, 13):
                
                # logging_in = True
                password = ""
                while True:
                    stdscr.clear()
                    stdscr.addstr(0, 0, "Login Screen")
                    stdscr.addstr(1, 0, "Password: " + password)

                    stdscr.refresh()
                    key = stdscr.getch()
                    if 32 <= key <= 126:
                        password += chr(key)
                    elif key == curses.KEY_BACKSPACE:
                        password = password[:-1]
                    elif key in (curses.KEY_ENTER, 10, 13):
                        client = try_make_client(password)
                        if client is not None:
                            logged_in = True
                            CLIENT = client
                            BALANCE = CLIENT.get_balance()
                            break
                        else:
                            show_message(stdscr, "Incorrect password.")
                    # password += str(key)
                
                # stdscr.getch()
        else:
            if key in (curses.KEY_ENTER, 10, 13):   
                if selected == 2:
                    SHOULD_KILL = True
                    return 
                
        if key == curses.KEY_UP and selected > 0:
            selected -= 1
        elif key == curses.KEY_DOWN and selected < len(menu) - 1:
            selected += 1
        # elif key in (curses.KEY_ENTER, 10, 13):
        #     if selected == 0:
        #         show_message(stdscr, "Balances:\n[...load from master here...]")
        #     elif selected == 1:
        #         show_message(stdscr, "Make transfer (not implemented)")
        #     elif selected == 2:
        #         break

def show_message(stdscr, text):
    stdscr.clear()
    lines = text.splitlines()
    for i, line in enumerate(lines):
        stdscr.addstr(i, 0, line)
    stdscr.addstr(len(lines) + 1, 0, "Press any key to go back...")
    stdscr.refresh()
    stdscr.getch()

if __name__ == "__main__":
    curses.wrapper(main)