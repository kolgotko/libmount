extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {

    println!("bindgen sys/mount.h");

    let bindings = bindgen::Builder::default()
        .header("/usr/include/sys/mount.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libc_mount.rs"))
        .expect("Couldn't write bindings!");

}
