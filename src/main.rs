use std::{process::exit, thread::sleep, time::Duration};

use crate::backend::bindings::{MasterOrderBook, OrderServer, close_master_server, open_master_server};


pub mod backend;

fn main() {


    let master = MasterOrderBook::new();

    println!("available regions: {:?}", MasterOrderBook::available_regions());


    let server = OrderServer::open(0, &master);
    println!("Started server on region: {}", server.get_name());

    server.open_record().unwrap();
    server.set_money(35).unwrap();
    server.flush_record().unwrap();

    sleep(Duration::from_millis(500));

    drop(server);

    sleep(Duration::from_millis(200));

    // let barrier = Barrier::new(2);


    // // server.try_lock().unwrap();
    // // server.try_lock().unwrap();

    // std::thread::scope(|s| {
        

    //     s.spawn(|| {
    //         loop {
    //             if server.try_lock(32).is_ok() {

    //                 server.release_lock(32);
    //             }
    //             // server.try_lock(32).unwrap();

    //             // soemthing
    //             // println!("user: {:?}", server.fetch_current_user());
       
    //             // server.release_lock(32);
    //         }
       
            

    //     });

    //     s.spawn(|| {
    //         loop {
                
    //             if server.try_lock(12).is_ok() {
    //                 let mut current = 12;
    //                 for i in 0..1000 {
    //                     let id = server.fetch_current_user();
    //                     if id != 12 {
    //                         println!("broken...");

    //                         server.open_record().unwrap();
    //                         server.set_money(35).unwrap();
    //                         server.flush_record().unwrap();
    //                         server.log_last_order();;
                            

    //                         current = id;
    //                         break;
    //                     }
                        
    //                 }   
    //                 server.release_lock(current);
    //             }
    //         }
            
    //     });


    // });
    // server.log_last_order();

    // server.log_last_order();

    // server.flush_record().unwrap();

    // server.log_last_order();;
    
    

    // let h = unsafe { &mut *open_server(3) };

    // println!("Hello, world! {}", unsafe { addsilly(3, 4) });
}
