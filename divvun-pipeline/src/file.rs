use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use log::{error, info};
use serde_json::Value;
use zip::ZipArchive;

use crate::pipeline::Pipeline;

pub static PIPELINE_EXTENSION: &'static str = "zpipe";
static JSON_EXTENSION: &'static str = "json";

#[derive(Debug)]
pub enum FileLoadError {
    NotAFile,
    InvalidExtension,
    NoStem,
    UnsupportedResource,
    NoJsonFile,
}

pub fn load_pipeline_file(pipeline_file: &str) -> Result<Pipeline, FileLoadError> {
    info!("Supplied file path: {}", pipeline_file);

    let pipeline_file = Path::new(pipeline_file);

    if !pipeline_file.is_file() {
        error!(
            "The supplied argument must be a valid file with the .{} extension",
            PIPELINE_EXTENSION
        );

        return Err(FileLoadError::NotAFile);
    }

    if pipeline_file.extension().is_none()
        || pipeline_file.extension().unwrap() != PIPELINE_EXTENSION
    {
        error!(
            "The supplied argument must be a valid file with the .{} extension",
            PIPELINE_EXTENSION
        );

        return Err(FileLoadError::InvalidExtension);
    }

    let file_stem = match pipeline_file.file_stem() {
        Some(file_stem) => file_stem,
        None => {
            error!("File stem missing");
            return Err(FileLoadError::NoStem);
        }
    };

    let parent = match pipeline_file.parent() {
        Some(parent) => {
            let mut new_parent = parent.to_path_buf();
            new_parent.push(file_stem);

            fs::create_dir_all(&new_parent).unwrap();
            Some(new_parent)
        }
        None => None,
    };

    let file = File::open(pipeline_file).unwrap();
    let reader = BufReader::new(&file);
    let mut archive = ZipArchive::new(reader).expect("zip");

    info!("File count: {}", archive.len());

    let mut json_file_option = None;
    let mut destination_dir = PathBuf::default();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let filename = file.sanitized_name();
        let ext = filename.extension().unwrap();

        if (&*file.name()).ends_with('/') {
            error!("Unexpected directory in zip file {:?} ignoring", filename);
            return Err(FileLoadError::UnsupportedResource);
        }

        if ext == JSON_EXTENSION {
            json_file_option = Some(filename.to_owned());

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

    let json_file;
    if json_file_option.is_none() {
        error!("No .json file found");
        return Err(FileLoadError::NoJsonFile);
    } else {
        json_file = json_file_option.unwrap();
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

    Ok(pipeline)
}
