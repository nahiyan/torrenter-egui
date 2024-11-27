use std::path::PathBuf;

fn main() {
    let dst_cxx = cmake::Config::new("cxx").build();
    println!("cargo:rustc-link-search=native={}", dst_cxx.display());

    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=static=torrent-rasterbar");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rerun-if-changed=./cxx/CMakeLists.txt");
    println!("cargo:rerun-if-changed=./cxx/api.h");
    println!("cargo:rerun-if-changed=./cxx/api.cpp");

    let bindings = bindgen::Builder::default()
        .header("cxx/api.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("./");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
