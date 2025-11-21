use cc::Build;




fn main() {
    
    println!("cargo::rerun-if-changed=backend/main.c");
    println!("cargo::rerun-if-changed=backend/main.h");
    println!("cargo::rerun-if-changed=backend/structs.h");
    println!("cargo::rerun-if-changed=backend");

    Build::new()
        .file("backend/main.c")
        .file("backend/signals/signal.c")
        .file("backend/helper/helper.c")
        .file("backend/master/master.c")
        .file("backend/channel/channel.c")
        .compile("backendlib");
    
}