
use crate::backend::server::TradingApplication;


pub mod backend;

fn main() {
    TradingApplication::start()
        .expect("The server has experienced a fatal error.");
}
