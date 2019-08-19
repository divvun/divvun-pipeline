use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
};

use log::{error, info};
use serde_json::Value;
use zip::ZipArchive;

use crate::{
    pipeline::Pipeline,
    resources::{LoadableResource, Resource, ResourceRegistry},
};

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

pub fn load_pipeline_file(
    pipeline_file: &Path,
) -> Result<(Pipeline, Arc<ResourceRegistry>), FileLoadError> {
    info!("Supplied file path: {}", pipeline_file.display());

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

    let resource_registry = Arc::new(ResourceRegistry::new());

    let file = File::open(pipeline_file).unwrap();
    let reader = BufReader::new(&file);
    let mut archive = ZipArchive::new(reader).expect("zip");

    info!("File count: {}", archive.len());

    let mut json_file_option = None;
    let mut destination_dir = PathBuf::default();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let filename = file.sanitized_name();
        info!("File {}: {:?}", i, filename);
        let ext = filename.extension();

        if (&*file.name()).ends_with('/') {
            error!("Unexpected directory in zip file {:?} ignoring", filename);
            return Err(FileLoadError::UnsupportedResource);
        }

        if ext.is_some() && ext.unwrap() == JSON_EXTENSION {
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
            let mut full_file_path = destination_dir.clone();
            full_file_path.push(filename.to_owned());

            info!("Loading resource path {:?}", full_file_path);
            let resource = LoadableResource::from(Resource::new_file(&full_file_path));

            let mut outfile = fs::File::create(&full_file_path).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();

            let str_filename = filename.to_str();

            match str_filename {
                Some(filename) => {
                    resource_registry.add_resource(filename, resource);
                    info!("Found resource file {:?}, adding to registry", filename);
                }
                None => {
                    error!("Mangled filename: {:?}", filename);
                    return Err(FileLoadError::UnsupportedResource);
                }
            };
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

    Ok((create_pipeline(json_file_path), resource_registry))
}

fn create_pipeline(json_file: PathBuf) -> Pipeline {
    info!("json_file_path: {:?}", json_file);
    let file = File::open(json_file).unwrap();
    let reader = BufReader::new(&file);

    let value: Value = serde_json::from_reader(reader).unwrap();

    let json_str = serde_json::to_string(&value).unwrap();

    let pipeline: Pipeline = Pipeline {
        root: serde_json::from_str(&json_str).unwrap(),
    };

    pipeline
}
