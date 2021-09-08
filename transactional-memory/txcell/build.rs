extern crate bindgen;

use std::env;
use std::path::PathBuf;

/// This build file generates Rust bindings for the libraries imported in
/// "wrapper.h", namely the LITMUS library, to use the functions
/// `be_migrate_thread_to_cpu` and `gettid` in tests.
///
/// See https://github.com/LITMUS-RT/liblitmus for documentation of LITMUS.
/// See https://rust-lang.github.io/rust-bindgen/ for documentation of bindgen.
fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=litmus");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Tell cargo to link the liblitmus library
    println!("cargo:rustc-link-search=/data/cnord/litmus/liblitmus");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg("-I/data/cnord/litmus/liblitmus/include")
        .clang_arg("-I/data/cnord/litmus/liblitmus/arch/x86/include")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
