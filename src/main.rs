use std::{process::exit, sync::Barrier, thread::sleep, time::{Duration, SystemTime}};

use crate::backend::{bindings::{MasterOrderBook, OrderServer, close_master_server, open_master_server}, server::TradingApplication};


pub mod backend;

fn main() {
    TradingApplication::start()
        .expect("The server has experienced a fatal error.");
}
