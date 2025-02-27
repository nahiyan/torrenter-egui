use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    // download libtorrent
    if !Path::new("libtorrent").exists() {
        println!("Downloading libtorrent.");
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                "v2.0.10",
                "--recurse-submodules",
                "https://github.com/arvidn/libtorrent.git",
            ])
            .status()
            .expect("Failed to download libtorrent");
    }

    // build and link cxx library
    let dst_cxx = cmake::Config::new("cxx").build();
    println!("cargo:rustc-link-search=native={}/lib", dst_cxx.display());
    println!("cargo:rustc-link-lib=static=cxx");

    // build and link libtorrent library
    let dst_libtorrent = cmake::Config::new("libtorrent")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_CXX_STANDARD", "17")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
    println!(
        "cargo:rustc-link-search=native={}/lib",
        dst_libtorrent.display()
    );
    println!("cargo:rustc-link-lib=static=torrent-rasterbar");

    // link indirect dependencies
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
