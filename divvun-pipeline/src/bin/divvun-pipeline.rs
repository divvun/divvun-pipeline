#![feature(async_await)]

use std::{env, fs::File, io::BufReader};

use clap::{crate_version, App, Arg};
use log::info;
use serde_json::Value;

use divvun_pipeline::{pipeline::Pipeline, run::run};

#[runtime::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let pipeline = "pipeline";

    let matches = App::new("divvun-pipeline")
        .version(crate_version!())
        .author("projektir <oprojektir@gmail.com>")
        .about("Asynchronous parallel pipeline for text processing.")
        .arg(
            Arg::with_name(pipeline)
                .help("The .zpipe file with the requested pipeline flow and required resources")
                .index(1),
        )
        .get_matches();

    if let Some(pipeline_file) = matches.value_of(pipeline) {
        info!("Offered file {}", pipeline_file);

        let file = File::open(pipeline_file).unwrap();
        let reader = BufReader::new(file);

        let value: Value = serde_json::from_reader(reader).unwrap();

        let json_str = serde_json::to_string(&value).unwrap();

        let pipeline: Pipeline = Pipeline {
            root: serde_json::from_str(&json_str).unwrap(),
        };

        let output = run(&pipeline).await;

        info!("Output: {}", &output);
    } else {
        info!("No file provided, skipping");
    }
}
