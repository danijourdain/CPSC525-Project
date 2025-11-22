//! This file manages the server code which
//! leverages the order books from the legacy C code.

use std::net::TcpListener;

use anyhow::Result;

use crate::backend::bindings::MasterOrderBook;



pub struct TradingApplication {
    master: MasterOrderBook
}

impl TradingApplication {
    pub fn start() -> Result<()> {

        let mut master = MasterOrderBook::new();
        master.open_order_server(0);
        master.open_order_server(1);
        master.open_order_server(2);

        // Start the listener on an address.
        let listener = TcpListener::bind("0.0.0.0:3402").unwrap();
        println!("log: started listener on {:?}", listener.local_addr()?);


        loop {
            
        }






        Ok(())
    }
}