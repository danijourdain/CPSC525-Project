use cc::Build;




fn main() {
    
    println!("cargo::rerun-if-changed=backend/main.c");

    Build::new()
        .file("backend/main.c")
        .compile("backendlib");
    
}