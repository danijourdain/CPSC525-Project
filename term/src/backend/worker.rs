use std::{io::{Read, Write}, net::TcpStream, sync::mpsc::{Receiver, RecvTimeoutError, Sender}, time::Duration};

use anyhow::Result;

use crate::gui::ledger::Trade;



/// The message that goes out from
/// the worker to the master.
pub enum ToWorkerMsg {
    /// A login attempt message.
    LoginAttempt(String),
    /// A transaction request.
    Trade {
        sender: usize,
        receiver: usize,
        money: usize
    }
}

/// The message that comes from the worker
/// and goes out to the master.
pub enum FromWorkerMsg {
    /// The connection is live.
    ConnectionLive,
    /// We have succesfully logged in.
    LoggedIn,
    /// We unlock the login (basically just an ack)
    LoginUnlock,
    /// The connection is dead.
    ConnectionDead,
    /// Update the application balance.
    Balance(i32),
    /// Update the order list.
    UpdateOrder(Vec<Trade>)
}



/// Runs the worker thread, this function is infallible
/// and surrounds the internal guarded one.
pub fn worker_thread(
    mut from_master: Receiver<ToWorkerMsg>,
    mut to_master: Sender<FromWorkerMsg>
) {

    loop {

        let _  = worker_thread_guarded(&mut from_master, &mut to_master);
    }
    

}

/// This is the fallible worker thread function.
fn worker_thread_guarded(
    from_master: &mut Receiver<ToWorkerMsg>,
    to_master: &mut Sender<FromWorkerMsg>
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
                        ToWorkerMsg::LoginAttempt(pwd) => {
                            if try_login(&mut stream, &pwd)? {
                                // Notify the GUI that we did log in.
                                let _ = to_master.send(FromWorkerMsg::LoggedIn);
                                password = Some(pwd);
                            } else {
                                // We failed to login.
                                password = None;
                            }
                            let _ = to_master.send(FromWorkerMsg::LoginUnlock);
                        }
                        ToWorkerMsg::Trade { sender, receiver, money } => {
                            if let Some(pwd) = password.clone() {
                                if try_login(&mut stream, &pwd)? {
                                    // Try to execute a trade.
                                    try_trade(sender, receiver, money, &mut stream)?;
                                } else {
                                    // The login failed.
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
                            // Update the balance and notify the GUI.
                            let balance = get_balance(&mut stream)?;
                            let _ = to_master.send(FromWorkerMsg::Balance(balance));

                            // Update the GUI code.
                            let orders = read_orders(&mut stream, 50)?;
                            let _ = to_master.send(FromWorkerMsg::UpdateOrder(orders));
                        } else {
                            // Failed, so we unset the passowrd.
                            password = None;
                            let _ = to_master.send(FromWorkerMsg::ConnectionDead);
                        }
                    }

                }
            }
            Err(_) => {
                // Do nothing.
                let _ = to_master.send(FromWorkerMsg::ConnectionDead).unwrap();
            }
        }


    }

    Ok(())
}


#[inline]
/// Reads a u32 from a stream.
pub fn read_u32(stream: &mut TcpStream) -> Result<u32> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}


/// Reads the orders from the stream, particularly the top-N
/// orders.
pub fn read_orders(stream: &mut TcpStream, top: usize) -> Result<Vec<Trade>> {
    // The buffer for trades.
    let mut buffer = vec![];

    // Write the discrminiator.
    stream.write_all(&[ 4u8 ])?;

    // Write the request.
    stream.write_all(&(top as u32).to_le_bytes())?;


    // Read the length.
    let mut length = [0u8; 4];
    stream.read_exact(&mut length).inspect_err(|_| println!("Error:"))?;
    let mut length = u32::from_le_bytes(length);

    while length > 0 {
        // Read the order details.
        let sender = read_u32(stream)?;
        let recipient = read_u32(stream)?;
        let money = read_u32(stream)?;


        // Read the order.
        buffer.push(Trade {
            money: money as usize,
            receiver: recipient as usize,
            sender: sender as usize
        });

        length -= 1;
    }

    // Flip the buffer.
    buffer.reverse();
    Ok(buffer)
}


/// Tries to perform a trade on the stream.
fn try_trade(sender: usize, receiver: usize, money: usize, stream: &mut TcpStream) -> Result<()> {
    // Form the payload, this
    // does not need an acknowledgement.
    let mut buffer = vec![ 2u8 ];
    buffer.extend_from_slice(&(sender as i32).to_le_bytes());
    buffer.extend_from_slice(&(receiver as i32).to_le_bytes());
    buffer.extend_from_slice(&(money as i32).to_le_bytes());
    stream.write_all(&buffer)?;

    Ok(())

}


/// Gets the balance from the stream using the
/// protocol.
fn get_balance(stream: &mut TcpStream) -> Result<i32> {
    stream.write_all(&[ 1u8 ])?; // Write the discriminator byte.
    
    let mut i32_buf = [0u8; 4]; // Read the integer.
    stream.read_exact(&mut i32_buf)?;
    Ok(i32::from_le_bytes(i32_buf))
}

/// Tries to log-in to the stream with a particular
/// password.
pub fn try_login(stream: &mut TcpStream, password: &str) -> Result<bool> {
    // Prepare and send the login payload.
    let mut payload = vec![ 0x00, 0x00 ];
    payload.extend_from_slice(&(password.len() as u32).to_le_bytes());
    payload.extend_from_slice(password.as_bytes());
    stream.write_all(&payload)?;


    let recv = &mut [ 0x00 ];
    stream.read_exact(recv)?;
    Ok(recv[0] == 1) // Wait for the ack byte.
}

