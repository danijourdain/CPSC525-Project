import curses

def main(stdscr):
    curses.curs_set(0)          # hide cursor
    stdscr.nodelay(False)       # blocking getch
    stdscr.keypad(True)         # enable arrows

    menu = ["View balances", "Make transfer", "Quit"]
    selected = 0

    while True:
        stdscr.clear()
        stdscr.addstr(0, 0, "=== Master Server Menu ===")

        for i, item in enumerate(menu):
            if i == selected:
                stdscr.attron(curses.A_REVERSE)
            stdscr.addstr(2 + i, 2, item)
            if i == selected:
                stdscr.attroff(curses.A_REVERSE)

        stdscr.refresh()

        key = stdscr.getch()
        if key == curses.KEY_UP and selected > 0:
            selected -= 1
        elif key == curses.KEY_DOWN and selected < len(menu) - 1:
            selected += 1
        elif key in (curses.KEY_ENTER, 10, 13):
            if selected == 0:
                show_message(stdscr, "Balances:\n[...load from master here...]")
            elif selected == 1:
                show_message(stdscr, "Make transfer (not implemented)")
            elif selected == 2:
                break

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