use std::path::PathBuf;

fn main() {
    // cc::Build::new().file("cxx/cat.c").compile("cxx");
    let dst_cxx = cmake::Config::new("cxx").build();
    println!("cargo:rustc-link-search=native={}", dst_cxx.display());
    // println!("cargo:rustc-link-lib=static=cxx");

    // TODO: Make it dynamic
    // let dst_libtorrent = cmake::Config::new("libtorrent")
    //     .define("BUILD_SHARED_LIBS", "OFF")
    //     .build();
    // println!(
    //     "cargo:rustc-link-search=native={}",
    //     dst_libtorrent.display()
    // );
    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=static=cxx");
    println!("cargo:rustc-link-lib=static=torrent-rasterbar");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rerun-if-changed=./cxx/CMakeLists.txt");
    // println!("cargo:rerun-if-changed=./libtorrent/CMakeLists.txt");
    println!("cargo:rerun-if-changed=./cxx/api.h");
    println!("cargo:rerun-if-changed=./cxx/api.cpp");

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("cxx/api.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from("./");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
