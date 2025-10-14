// Build script - flutter_rust_bridge will auto-generate bindings
fn main() {
    // For flutter_rust_bridge v2, bindings are generated via CLI tool
    // Run: flutter_rust_bridge_codegen generate
    // This is done manually, not in build.rs
    println!("cargo:rerun-if-changed=src/lib.rs");
}
