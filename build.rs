use std::path::PathBuf;

fn main() {
    // cxx library
    let dst_cxx = cmake::Config::new("cxx").build();
    println!("cargo:rustc-link-search=native={}", dst_cxx.display());
    println!("cargo:rustc-link-lib=static=cxx");

    // libtorrent library
    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=static=torrent-rasterbar");

    // indirect dependencies
    // println!("cargo:rustc-link-lib=asan");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=stdc++");

    // files to watch for changes
    println!("cargo:rerun-if-changed=./cxx/CMakeLists.txt");
    println!("cargo:rerun-if-changed=./cxx/api.h");
    println!("cargo:rerun-if-changed=./cxx/api.cpp");

    // generate the bindings
    let bindings = bindgen::Builder::default()
        .header("cxx/api.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("./");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
