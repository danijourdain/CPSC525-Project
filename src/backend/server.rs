//! This file manages the server code which
//! leverages the order books from the legacy C code.

use std::{
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread::{sleep, spawn},
    time::Duration,
};

use anyhow::{Result, anyhow};

use crate::backend::bindings::{AcquiredOrderServer, MasterOrderBook};

pub struct TradingApplication;

impl TradingApplication {
    pub fn start() -> Result<()> {
        start_app()
    }
}

// fn read_message(
//     state: Arc<MasterOrderBook>,
//     stream: TcpStream,
//     addr: SocketAddr
// ) -> Result<()> {

//     Ok(())
// }

fn read_u8(stream: &mut TcpStream) -> std::io::Result<u8> {
    let mut buffer = [0];
    stream.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

fn read_u32(stream: &mut TcpStream) -> std::io::Result<u32> {
    let mut buffer = [0, 0, 0, 0];
    stream.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_i32(stream: &mut TcpStream) -> std::io::Result<i32> {
    let mut buffer = [0, 0, 0, 0];
    stream.read_exact(&mut buffer)?;
    Ok(i32::from_le_bytes(buffer))
}

fn read_string(stream: &mut TcpStream) -> Result<String> {
    let len = read_u32(stream)? as usize;

    let mut vec = vec![0; len];
    stream.read_exact(&mut vec)?;
    Ok(std::str::from_utf8(vec.as_ref())?.to_string())
}

fn write_u8(stream: &mut TcpStream, value: u8) -> std::io::Result<()> {
    stream.write_all(&[value])?;
    Ok(())
}

fn write_i32(stream: &mut TcpStream, value: i32) -> std::io::Result<()> {
    stream.write(&value.to_le_bytes())?;
    Ok(())
}

fn conn_handler(
    state: Arc<MasterOrderBook>,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    let mut auth: Option<AcquiredOrderServer<'_>> = None;

    loop {
        let delimiter = read_u8(&mut stream)?;
        if delimiter == 0 {
            // This is a login message.
            let region = read_u8(&mut stream)? as i32;
            let password = read_string(&mut stream)?;

            // println!("Trying with password: {password}");

            let Some(server) = state.get_region_server(region) else {
                return Err(anyhow!("failed to retrieve region ID."));
            };

            loop {
                match server.try_lock(&password) {
                    Ok(server) => {
                        // println!("server");
                        auth = Some(server);
                        write_u8(&mut stream, 1)?;
                        break;
                    }
                    Err(e) => {
                        // println!("fail");
                        if e.kind() == ErrorKind::ResourceBusy {
                            continue;
                        } else {
                            write_u8(&mut stream, 0)?;
                            break;
                        }
                    }
                }
            }
        } else if delimiter == 1 {
            match &auth {
                Some(obj) => {
                    // println!("HEY");
                    write_i32(&mut stream, obj.get_balance())?;
                }
                None => {
                    // the client requested balance data w/o being logged in.
                    return Err(anyhow!("client used bad sequence."));
                }
            }
        } else if delimiter == 2 {
            match &auth {
                Some(obj) => {
                    let sender = read_i32(&mut stream)?;
                    let recipient = read_i32(&mut stream)?;
                    let money = read_i32(&mut stream)?;

                    // Open and submit a record.
                    obj.open_record()?;
                    obj.set_sender(sender)?;
                    obj.set_recipient(recipient)?;
                    obj.set_money(money)?;
                    obj.flush_record()?;

                    println!("log: transacted. sender={sender}, recipient={recipient}, money={money}");
                }
                None => {
                    // the client requested balance data w/o being logged in.
                    return Err(anyhow!("client used bad sequence."));
                }
            }
        } else {
            println!("log(error): client used weird delimiter: {delimiter}");
        }
    }
}

fn start_app() -> Result<()> {
    // Start up the order book.
    let mut master = MasterOrderBook::new();
    master.open_order_server(0);
    master.open_order_server(1);
    master.open_order_server(2);
    let master = Arc::new(master);

    // Start the listener on an address.
    let listener = TcpListener::bind("0.0.0.0:3402").unwrap();
    println!("log: started listener on {:?}", listener.local_addr()?);

    loop {
        // Accept a new connection.
        let (stream, addr) = listener.accept()?;

        // Spawn a thread to handle the connection
        spawn({
            let master = Arc::clone(&master);
            move || conn_handler(master, stream, addr)
        });
    }
}
