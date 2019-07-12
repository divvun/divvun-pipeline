#![feature(async_await)]

use std::{env, fs};

use divvun_pipeline::{file::load_pipeline_file, run::run};

use divvun_schema::{capnp_message, string_capnp::string};

#[runtime::test]
async fn pipeline_run_with_zpipe() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let msg = capnp_message!(string::Builder, builder => {
        builder.set_string("Hello world!");
    });

    let msg_vec = divvun_schema::util::message_to_vec(msg).unwrap();

    let (pipeline, registry) = load_pipeline_file("tests/pipeline.zpipe").unwrap();
    let output = run(pipeline, registry, msg_vec).await;

    assert_eq!(
        "EREH ENOD SNOITATUPMOC GIB AHello world!\n😋\n!ymmuy",
        output
    );

    fs::remove_dir_all("tests/pipeline").expect("failed to remove test pipeline directory");
}
