use std::{env, path::PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir);
    let hfst_dir = PathBuf::from("hfst");

    let hfst_include_dir = hfst_dir.join("include").canonicalize().unwrap();
    let hfst_lib_dir = hfst_dir.join("lib").canonicalize().unwrap();

    println!("cargo:rustc-link-search=native={}", hfst_lib_dir.display());
    println!("cargo:rustc-link-lib=hfst");

    cc::Build::new()
        .file("wrapper/wrapper.cpp")
        .include(hfst_include_dir.clone())
        .include(hfst_include_dir.join("hfst"))
        .static_flag(true)
        .cpp(true)
        .flag("-std=c++11")
        .compile("hfst_wrapper");

    let bindings = bindgen::Builder::default()
        .header("wrapper/wrapper.hpp")
        .clang_arg(format!("-I{}", hfst_include_dir.display()))
        .clang_arg(format!("-I{}", hfst_include_dir.join("hfst").display()))
        .clang_arg("-std=c++14")
        .clang_arg("-xc++")
        .opaque_type("std::.*")
        .whitelist_type("hfst_ol_tokenize::TokenizeSettings")
        .generate()
        .expect("unable to generate hfst bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
