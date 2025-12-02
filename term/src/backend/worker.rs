use std::{io::{Read, Write}, net::TcpStream, sync::mpsc::{Receiver, RecvTimeoutError, Sender}, time::Duration};

use anyhow::Result;



pub enum ToWorkerMessageContents {
    LoginAttempt(String),
    Trade {
        sender: usize,
        receiver: usize,
        money: usize
    }
}


pub enum FromWorkerMsg {
    ConnectionLive,
    LoggedIn,
    LoginUnlock,
    ConnectionDead,
    Balance(i32)
}

// pub struct ToWorkerMessage {
//     ack: Sender<()>,
//     contents: ToWorkerMessageContents
// }




pub fn worker_thread(
    mut from_master: Receiver<ToWorkerMessageContents>,
    mut to_master: Sender<FromWorkerMsg>
) {

    loop {
        let _  =worker_thread_guarded(&mut from_master, &mut to_master);
    }
    

}

// self.server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
//         self.server.connect(self.addr)
        
//         password = self.password.encode()
        
//         self.server.sendall(b'\0' + int.to_bytes(self.region, byteorder='little', length=1) + int.to_bytes(len(password), length=4, byteorder='little') + password)
        
//         if self.server.recv(1) == b'\1':
//             return True
//         else:
//             return False

fn worker_thread_guarded(
    mut from_master: &mut Receiver<ToWorkerMessageContents>,
    mut to_master: &mut Sender<FromWorkerMsg>
) -> Result<()> {


    let mut password: Option<String> = None;
    let _ = to_master.send(FromWorkerMsg::LoginUnlock);

    loop {
        let command = match from_master.recv_timeout(Duration::from_millis(50)) {
            Ok(v) => Some(v),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break 
        };
        match TcpStream::connect("0.0.0.0:3402") {
            Ok(mut stream) => {
                let _ = to_master.send(FromWorkerMsg::ConnectionLive).unwrap();

                if let Some(command) = command {
                    match command {
                        ToWorkerMessageContents::LoginAttempt(pwd) => {
                            // println!("Starting... ({password})");
                            if try_login(&mut stream, &pwd).inspect_err(|e| println!("failed"))? {
                                let _ = to_master.send(FromWorkerMsg::LoggedIn);
                                password = Some(pwd);
                            } else {
                                // We failed to login.
                                password = None;
                            }
                            // println!("Done...");
                            let _ = to_master.send(FromWorkerMsg::LoginUnlock);
                        }
                        ToWorkerMessageContents::Trade { sender, receiver, money } => {
                            if let Some(pwd) = password.clone() {
                                if try_login(&mut stream, &pwd)? {
                                    try_trade(sender, receiver, money, &mut stream)?;
                                } else {
                                    password = None;
                                    let _ = to_master.send(FromWorkerMsg::ConnectionDead);
                                }
                            }
                        }
                    }
                } else {
                    if let Some(pwd) = password.clone() {
                        // we have the password.
                        if try_login(&mut stream, &pwd)? {
                            // Logged in.
                            // println!("Accessed.");

                            let balance = get_balance(&mut stream)?;
                            let _ = to_master.send(FromWorkerMsg::Balance(balance));

                        } else {
                            password = None;
                            let _ = to_master.send(FromWorkerMsg::ConnectionDead);
                        }
                    }

                }
            }
            Err(e) => {
                // Do nothing.
                let _ = to_master.send(FromWorkerMsg::ConnectionDead).unwrap();
            }
        }

        // println!("Hi");

    }

    Ok(())
}


// self.server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
//         self.server.connect(self.addr)
        
//         password = self.password.encode()
        
//         self.server.sendall(b'\0' + int.to_bytes(self.region, byteorder='little', length=1) + int.to_bytes(len(password), length=4, byteorder='little') + password)
        
//         if self.server.recv(1) == b'\1':
//             return True
//         else:
//             return False


fn try_trade(sender: usize, receiver: usize, money: usize, stream: &mut TcpStream) -> Result<()> {
    // self.server.sendall(b'\2' + int.to_bytes(self.region, length=4, byteorder='little', signed=True) + int.to_bytes(recipient, length=4, byteorder='little', signed=True) + int.to_bytes(money, length=4, byteorder='little', signed=True))


    let mut buffer = vec![ 2u8 ];
    buffer.extend_from_slice(&(sender as i32).to_le_bytes());
    buffer.extend_from_slice(&(receiver as i32).to_le_bytes());
    buffer.extend_from_slice(&(money as i32).to_le_bytes());
    stream.write_all(&buffer)?;

    Ok(())

}

fn get_balance(stream: &mut TcpStream) -> Result<i32> {
    stream.write_all(&[ 1u8 ])?;
    
    let mut i32_buf = [0u8; 4];
    stream.read_exact(&mut i32_buf)?;
    Ok(i32::from_le_bytes(i32_buf))
}

fn try_login(stream: &mut TcpStream, password: &str) -> Result<bool> {


    let mut payload = vec![ 0x00, 0x00 ];
    // payload.extend_from_slice(&[ 0u8 ]);
    payload.extend_from_slice(&(password.len() as u32).to_le_bytes());
    payload.extend_from_slice(password.as_bytes());
    stream.write_all(&payload)?;


    let mut recv = &mut [ 0x00 ];
    stream.read_exact(recv)?;
    Ok(recv[0] == 1)

    // password = self.password.encode()
        
    //     self.server.sendall(b'\0' + int.to_bytes(self.region, byteorder='little', length=1) + int.to_bytes(len(password), length=4, byteorder='little') + password)
        
    //     if self.server.recv(1) == b'\1':
    //         return True
    //     else:
    //         return False
        
}

fn run_live_connection(
    from_master: &mut Receiver<ToWorkerMessageContents>,
    to_master: &mut Sender<FromWorkerMsg>,
    stream: TcpStream
) -> Result<()> {
    loop {
        match from_master.recv_timeout(Duration::from_millis(100)) {
            Ok(v) => {

            }
            Err(e) => match e {
                RecvTimeoutError::Disconnected => break,
                RecvTimeoutError::Timeout => { /* Nothing to do! */}
            }
        }
    }

    Ok(())
}