use std::{process::exit, sync::Barrier, thread::sleep, time::{Duration, SystemTime}};

use crate::backend::bindings::{MasterOrderBook, OrderServer, close_master_server, open_master_server};


pub mod backend;

fn main() {

    // std::fs::remove_file("database.csv").unwrap();


    // let master = MasterOrderBook::new();

    // println!("available regions: {:?}", MasterOrderBook::available_regions());


    // let server = OrderServer::open(0, &master);
    // println!("Started server on region: {}", server.get_name());

    // let time = SystemTime::now();
    // server.try_lock("bluecircle123").unwrap();
    // println!("Elapsed: {:?}", time.elapsed().unwrap());

    // server.open_record().unwrap();
    // server.set_money(35).unwrap();
    // server.flush_record().unwrap();

    // sleep(Duration::from_millis(500));

    // drop(server);

    // sleep(Duration::from_millis(200));


     let master = MasterOrderBook::new();
     let server = OrderServer::open(0, &master);

    let barrier = Barrier::new(2);

    server.open_record().unwrap();
    server.set_sender(0).unwrap();
    server.set_recipient(1).unwrap();
    server.set_money(35).unwrap();
    server.flush_record().unwrap();


    if false {
        

    const THRESH: usize = 1;

    // server.try_lock().unwrap();
    // server.try_lock().unwrap();

    std::thread::scope(|s| {
        

        s.spawn(|| {
            let mut i = 0;
            let mut secondary = 0;
            let mut acc = 0;
            loop {
                // println!("Trying lock at: {:?}", SystemTime::now());
                if server.try_lock("bluecircle123").is_ok() {
                    acc += 1;
                    sleep(Duration::from_millis(100));
                    server.release_lock(32);
                }


                if i == THRESH {

                    println!("hey... {secondary} {acc}");
                    secondary += 1;
                    i = 0;
                }
                
                i += 1;

                // server.try_lock(32).unwrap();

                // soemthing
                // println!("user: {:?}", server.fetch_current_user());
       
                // server.release_lock(32);
            }
        });

        s.spawn(|| {
            let mut i = 0;
            let mut secondary = 0;
            let mut acc =0 ;
            loop {
                // println!("Adversary tyring at @ {:?}", SystemTime::now());
                if server.try_lock("susss").is_ok() {
                    // let mut current = 12;
                    // for i in 0..1000 {
                    //     let id = server.fetch_current_user();
                    //     if id != 12 {
                    //         println!("broken...");

                    //         // server.open_record().unwrap();
                    //         // server.set_money(35).unwrap();
                    //         // server.flush_record().unwrap();
                    //         // server.log_last_order();;
                            

                    //         current = id;
                    //         break;
                    //     }
                        
                    // }  

                    server.open_record().unwrap();
            
                    server.set_money(30).unwrap();

                    server.flush_record().unwrap();
                    // println!("BROKEN...");
                    // sleep(Duration::from_millis(100));
                    acc += 1;
                    // println!("Broken....");
                    server.release_lock(12);
                }

                if i == THRESH {

                    println!("wow... {secondary} {acc}");
                    secondary += 1;
                    i = 0;
                }
                
                i += 1;
            }
            
        });


    });
    }
    // server.log_last_order();

    // server.log_last_order();

    // server.flush_record().unwrap();

    // server.log_last_order();;
    
    

    // let h = unsafe { &mut *open_server(3) };

    // println!("Hello, world! {}", unsafe { addsilly(3, 4) });
}
