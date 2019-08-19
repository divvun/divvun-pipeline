#![feature(async_await)]

use std::{
    env,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use clap::{crate_version, App, Arg};
use log::{error, info};

use divvun_pipeline::{
    file::{load_pipeline_file, PIPELINE_EXTENSION},
    run::PipelinRunConfigurationBuilder,
    module::AllocationType
};

#[runtime::main]
async fn main() {
    env_logger::init();

    let pipeline = "pipeline";

    let matches = App::new("divvun-pipeline")
        .version(crate_version!())
        .author("projektir <oprojektir@gmail.com>")
        .about("Asynchronous parallel pipeline for text processing.")
        .arg(
            Arg::with_name(pipeline)
                .help(&format!(
                    "The .{} file with the requested pipeline flow and required resources",
                    PIPELINE_EXTENSION
                ))
                .index(1),
        )
        .arg(
            Arg::with_name("modules")
                .help("Modules search path")
                .short("m")
                .takes_value(true),
        )
        .get_matches();

    let mut vec_buffer = Vec::new();
    io::stdin().read_to_end(&mut vec_buffer).ok();
    info!("Input size: {}", vec_buffer.len());

    if vec_buffer.len() == 0 {
        error!("No input received");
        return;
    }

    if let Some(pipeline_file) = matches.value_of(pipeline) {
        match load_pipeline_file(Path::new(pipeline_file)) {
            Ok((pipeline, resources, _td)) => {
                let mut builder = PipelinRunConfigurationBuilder::default()
                    .pipeline(pipeline)
                    .resources(resources)
                    .input(vec_buffer);

                if let Some(search_path) = matches.value_of("modules") {
                    builder = builder.module_search_path(PathBuf::from(search_path));
                }

                let runner = builder.build().expect("failed to build pipeline runner");
                let mut result = runner.run().await;
                io::copy(&mut result.output, &mut io::stdout()).expect("write to succeed");
            }
            Err(e) => {
                error!("Error loading pipeline file: {:?}", e);
                return;
            }
        }
    }
}
