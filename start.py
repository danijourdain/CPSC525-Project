import os
import subprocess
import sys

def run_tmux(*args):
    """Run a tmux command, raising if it fails"""
    try:
        subprocess.run(["tmux", *args], check=True)
    except FileNotFoundError:
        print("Error: tmux not found", file=sys.stderr)
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print(f"Error: tmux command failed: {' '.join(e.cmd)}", file=sys.stderr)
        sys.exit(1)


def main():
    # Use the directory where this script is as project root.
    proj_root = os.path.dirname(os.path.abspath(__file__))

    session_name = "demo"

    # if a session named "demo" already exists, kill it and start fresh
    has = subprocess.run(
        ["tmux", "has-session", "-t", session_name],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if has.returncode == 0:
        # Session exists kill it
        subprocess.run(
            ["tmux", "kill-session", "-t", session_name],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    # 1. Create a detached tmux session with single window as one pane
    # Start in proj_root.
    run_tmux(
        "new-session",
        "-d",
        "-s",
        session_name,
        "-c",
        proj_root,
        "bash" # shell to start the pane
    )

    # we have window 0, pane 0 Top pane is app Terminal
    # split vertically to create a bottom pane (pane 1)
    run_tmux(
        "split-window",
        "-v",
        "-t",
        f"{session_name}:0",
        "-c",
        proj_root,
    )

    # split bottom pane
    run_tmux(
        "select-pane",
        "-t",
        f"{session_name}:0.1"
    )
    run_tmux(
        "split-window",
        "-h",
        "-t",
        f"{session_name}:0.1",
        "-c",
        proj_root,
    )

    # send commands to each pane
    # bottom-left: The Server
    # run 'cargo run' from project root
    run_tmux(
        "send-keys",
        "-t",
        f"{session_name}:0.1",
        "cargo run",
        "C-m",
    )

    # Top pane: The Terminal
    # cd into term and cargo run
    run_tmux(
        "send-keys",
        "-t",
        f"{session_name}:0.0",
        "cd term && cargo run",
        "C-m",
    )

    # Bottom-right pane: The Attack
    # Prepare the attack commands press Enter when ready to start attack
    run_tmux(
        "send-keys",
        "-t",
        f"{session_name}:0.2",
        "echo '>>> Start the attack AFTER logging in to the application with password. Press ENTER when ready.'",
        "C-m",
    )
    run_tmux(
        "send-keys",
        "-t",
        f"{session_name}:0.2",
        "python attack/attack.py",
    )

    # Focus top pane default
    run_tmux(
        "select-pane",
        "-t",
        f"{session_name}:0.0",
    )

    # attach to session so user sees the layout
    run_tmux(
        "attach-session",
        "-t",
        session_name
    )

if __name__ == "__main__":
    main()