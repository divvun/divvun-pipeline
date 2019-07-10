#![feature(async_await)]

use std::{env, fs::File, io::BufReader};

use serde_json::Value;

use divvun_pipeline::{pipeline::Pipeline, run::run};

#[runtime::test]
async fn pipeline_run_with_json() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let file = File::open("pipeline.json").unwrap();
    let reader = BufReader::new(file);

    let value: Value = serde_json::from_reader(reader).unwrap();

    let json_str = serde_json::to_string(&value).unwrap();

    let pipeline: Pipeline = Pipeline {
        root: serde_json::from_str(&json_str).unwrap(),
    };

    let result = run(&pipeline).await;

    assert_eq!("EREH ENOD SNOITATUPMOC GIB AHello world!", result);
}
