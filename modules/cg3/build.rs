use std::{env, path::PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir);
    let cg3_dir = PathBuf::from("cg3");

    let cg3_include_dir = cg3_dir.join("include").canonicalize().unwrap();
    let cg3_lib_dir = cg3_dir.join("lib").canonicalize().unwrap();

    println!("cargo:rustc-link-search=native={}", cg3_lib_dir.display());
    println!("cargo:rustc-link-lib=cg3");

    cc::Build::new()
        .file("wrapper/wrapper.cpp")
        .include(cg3_include_dir.clone())
        .include(cg3_include_dir.join("cg3"))
        .static_flag(true)
        .cpp(true)
        .flag("-std=c++11")
        .compile("cg3_wrapper");

    // let bindings = bindgen::Builder::default()
    //     .header("wrapper/wrapper.hpp")
    //     .clang_arg(format!("-I{}", cg3_include_dir.display()))
    //     // .clang_arg(format!("-I{}", cg3_include_dir.join("hfst").display()))
    //     // .clang_arg("-x c++")
    //     .clang_arg("-std=c++11")
    //     .opaque_type("std::.*")
    //     .whitelist_type("hfst_ol_tokenize::TokenizeSettings")
    //     .generate()
    //     .expect("unable to generate hfst bindings");

    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings!");
}
