# CPSC525-Project

Our CWE was race conditions. This README.md will describe the nature of the vulnerability and the explopit.

# The Application
Our application is a trading system. The trading system works by having a central order book, called the "Master order book"
which runs on a unique thread. The master order book is the one that actually pushes to the disk and thus is the important one. There
are then "subjugate books" which represent the books for a particular region.

When someone logs into the system, they connect to their subjugate regional order book which enables them to place orders conditional on the fact that they have the correct password, and thus, are able to actually gain access to the server.

## Regional Passwords
Each region has a password. For the sake of simplicity, the GUI only works with the Calgary region as of now. Here are the passwords for each region:
- Calgary: `bluecircle123`
- New York: `imwalkinghere`
- Signapore: `signaporerules`


## Note
A fictional worker named Jim sometimes will appear in the C code, he is partially responsible for the exploit. He would be considered to be one of the developers who worked on the original legacy `C` backend. Please note that it is actually not technically the fault of the legacy code designers, since they were designing library code. This could have been easily prevented in the Rust integration.

# Vulnerability
- To get access to the order book you need to login which invokes the `try_lock` function (`backend/main.c`) which processes the request and is supposed to only handle one request at a time.
- The server in normal operation uses a very extensive hashing system where SHA256 is used iteratively in order to improve the security of the system theoretically. However, in high traffic, we do not care as much as this level is already excessive, so we lower it by one, perform the check, and then raise it so we do not forget what the level was at.
- This is all fine and good since the level should always reset, HOWEVER, this is not actually the case! As it turns out, since there is no mutex protecting the critical section, we have the case where the security level can get lowered twice, bringing it to zero and putting it in dev mode where the security is turned off!
- This is trivially fixed by the introduction of a mutex that protects the critical section/

# Exploit
- An authenticated user logs in on their GUI and leaves it running. As per the protocol, each request requires a new login (this is to prevent people holding the lock for a long period of time) and so the GUI applicaiton constantly checks the balance.
- An attacker, who does not know the password, repeatedly tries. After enough time, high traffic mode is triggered and with enough attempts the executions line up enough to set the security level lower, where it is eventually set to zero, corresponding to `SEC_NONE` and allowing the intruder through! Now they can execute an order maliciously.

# The Fix
As discussed in the prior sections, the most trivial fix is just to use a Mutex to protect the critical section.


# Running the Code
The code has three parts: the **server**, the **client**, and the **attack**. We can compile each as follows, keeping in mind that all of these should be executed from the main directory.

## The Server
The easiest way to run the server is to simply invoke a cargo run:
```bash
$ cargo run
```

## The Terminal
This is compiled the same as the server, except we must navigate to the `term/` directory first:
```bash
$ cd term
$ cargo run
```

## The Attack
The attack can be run from the main directory.
```bash
$ python attack/attack.py
```

## How to Perform the Attack?
First, you must start the server. Then you must start the GUI application. Once you have started the GUI application and logged in with the password `bluecircle123` you can start the attack. After a while it will say `ACCESSED!` meaning that it has succesfully inserted the malicious order in the book.


# Appendix

## Use of Multiple Languages
- We used `C` for the legacy backend.
- We used `Rust` for two things:
    - The modern trading server, which uses `C` through special bindings.
    - The GUI trading application, which runs wholly in Rust.
- We used `Python` for the exploit, as a real hacker may do.

## Directories
There are several components of this project:
1. `attack/` contains the attack files. There are two subfiles.
    1. `attack.py` contains a script that executes the exploit and transfers $38 to New York.
    2. `client.py` is the client library which enables us to connect to the server.
2. `backend/` contains the code for the "legacy" C backend.
3. `src/` contains the source code for the Rust integration.
4. `term/` contains the Rust application for the trading GUI.

## Elaboration on Rust Fault
Rust often claims "fearless concurrency" so why did the Rust integration not catch the error? Well, in Rust occasionally we need to assert things to the compiler if we know what we are doing. For instance, to implement a Mutex we need to tell the compiler that what we are doing is actually thread safe, since it does not know that yet. The following allows the master order book to be accessed across threads:
```rust
// We assume that these are thread safe, which
// introduces the vulnerability. This is unsound and
// is partially what allows the vulnerability to take place.
unsafe impl Send for MasterOrderBook {}
unsafe impl Sync for MasterOrderBook {}
```


