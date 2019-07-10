#![feature(async_await)]

use std::fs::File;
use std::io::BufReader;
use std::env;

use serde_json::Value;

use divvun_pipeline::pipeline::Pipeline;
use divvun_pipeline::run::run;

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
