#![feature(async_await)]

use std::fs::{self, File};
use std::io::{self, BufReader};
use std::env;
use std::path::{Path, PathBuf};

use zip::ZipArchive;
use log::{info, error};
use clap::{App, Arg, crate_version};
use serde_json::Value;

use divvun_pipeline::{pipeline::Pipeline, run::run};

// TODO: this shouldn't live here
static PIPELINE_EXTENSION: &'static str = "zpipe";
static JSON_EXTENSION: &'static str = "json";

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
                .help(&format!("The .{} file with the requested pipeline flow and required resources", PIPELINE_EXTENSION))
                .index(1),
        )
        .get_matches();

    // TODO: Consolidate file processing
    if let Some(pipeline_file) = matches.value_of(pipeline) {
        info!("Supplied file path: {}", pipeline_file);

        let pipeline_file = Path::new(pipeline_file);

        if !pipeline_file.is_file() || (pipeline_file.extension().is_none() || pipeline_file.extension().unwrap() != PIPELINE_EXTENSION) {
            error!("The supplied argument must be a valid file with the .{} extension", PIPELINE_EXTENSION);
            return;
        }

        let file_stem = match pipeline_file.file_stem() {
            Some(file_stem) => {
                file_stem
            },
            None => {
                error!("Invalid file supplied: {}", pipeline_file.display());
                return;
            }
        };

        let parent = match pipeline_file.parent() {
            Some(parent) => {
                let mut new_parent = parent.to_path_buf();
                new_parent.push(file_stem);

                fs::create_dir_all(&new_parent).unwrap();
                Some(new_parent)
            },
            None => None,
        };

        let file = File::open(pipeline_file).unwrap();
        let reader = BufReader::new(&file);
        let mut archive = ZipArchive::new(reader).expect("zip");

        info!("File count: {}", archive.len());

        let mut json_file= PathBuf::default();
        let mut destination_dir = PathBuf::default();

        for i in 0 .. archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let filename = file.sanitized_name();
            let ext = filename.extension().unwrap();

            if (&*file.name()).ends_with('/') {
                error!("Unexpected directory in zip file {:?} ignoring", filename);
                return;
            }

            if ext == JSON_EXTENSION {
                json_file = filename.to_owned();

                let cloned_parent = parent.clone();
                if cloned_parent.is_some() {
                    destination_dir.push(cloned_parent.unwrap());
                }

                let mut temp_destination = destination_dir.clone();
                temp_destination.push(filename.clone());
                let mut outfile = fs::File::create(&temp_destination).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();

                info!("Found {:?}, extracting", filename);
            } else {
                info!("Unexpected file, ignoring: {:?}", filename);
            }
        }

        let mut json_file_path = destination_dir.clone();
        json_file_path.push(json_file);

        info!("json_file_path: {:?}", json_file_path);
        let file = File::open(json_file_path).unwrap();
        let reader = BufReader::new(&file);

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
