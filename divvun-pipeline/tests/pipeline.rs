#![feature(async_await)]

use std::{env, fs};

use divvun_pipeline::{file::load_pipeline_file, run::run};

#[runtime::test]
async fn pipeline_run_with_zpipe() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let pipeline = load_pipeline_file("tests/pipeline.zpipe").unwrap();
    let output = run(&pipeline).await;

    assert_eq!("EREH ENOD SNOITATUPMOC GIB AHello world!", output);

    fs::remove_dir_all("tests/pipeline").expect("failed to remove test pipeline directory");
}
