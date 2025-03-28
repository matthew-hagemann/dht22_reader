extern crate bindgen;

use std::fs;
use std::path::PathBuf;

#[cfg(feature = "generate-bindings")]
fn generate_bindings() {
    // Use bindgen to generate the bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_copy(false)
        .generate_block(false)
        .layout_tests(false)
        .derive_debug(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src/bindings");
    fs::create_dir_all(&out_path).expect("Couldn't create bindings directory");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    // Tell cargo to rerun build if any of the included headers change
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=gpiod");

    #[cfg(feature = "generate-bindings")]
    generate_bindings();
}
