#![feature(async_await)]

use divvun_pipeline::{file::load_pipeline_file, run::PipelinRunConfigurationBuilder};
use divvun_schema::{capnp_message, string_capnp::string};
use std::{env, fs, path::PathBuf};

mod common;

#[runtime::test]
async fn pipeline_run_with_zpipe() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let msg = capnp_message!(string::Builder, builder => {
        builder.set_string("Hello world!");
    });

    let msg_vec = divvun_schema::util::message_to_vec(msg).unwrap();

    let mut pipeline_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pipeline_file.push("tests/pipeline.zpipe");
    let (pipeline, registry, _td) = load_pipeline_file(&pipeline_file).unwrap();
    let runner = PipelinRunConfigurationBuilder::default()
        .pipeline(pipeline)
        .resources(registry)
        .input(msg_vec)
        .module_search_path(common::get_test_module_search_path())
        .build()
        .unwrap();
    let output = runner.run().await;

    assert_eq!(
        "EREH ENOD SNOITATUPMOC GIB AHello world!\n😋\n!ymmuy",
        output
    );
}
