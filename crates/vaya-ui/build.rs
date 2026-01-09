//! Build script for vaya-ui
//!
//! Generates WASM-compatible types from vaya-common during compilation.

fn main() {
    // Rerun if the source changes
    println!("cargo:rerun-if-changed=../vaya-common/src/types.rs");
    println!("cargo:rerun-if-changed=../vaya-common/src/codegen/mod.rs");

    // Generate frontend types
    let output_path = std::path::Path::new("src/generated_types.rs");

    // Only generate if not already present or if source is newer
    if let Err(e) = vaya_common::codegen::generate_frontend_types(
        output_path.to_str().expect("valid path"),
    ) {
        println!("cargo:warning=Failed to generate frontend types: {}", e);
        // Don't fail the build - the types might already exist
    } else {
        println!("cargo:note=Generated frontend types at {:?}", output_path);
    }
}
