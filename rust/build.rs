fn main() {
    // flutter_rust_bridge_codegen will handle code generation
    // This is invoked automatically during build
    println!("cargo:rerun-if-changed=src/api.rs");
}
