use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::{tempdir, TempDir};

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
    NoTempDir,
    UnsupportedResource,
    NoJsonFile,
}

pub fn load_pipeline_file(
    pipeline_file: &Path,
) -> Result<(Pipeline, Arc<ResourceRegistry>, TempDir), FileLoadError> {
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

    // Temporary dir to extract archive to
    let temp_target_dir = match tempdir() {
        Ok(dir) => dir,
        Err(_) => {
            error!("Failed to create temporary directory");
            return Err(FileLoadError::NoTempDir);
        }
    };

    let resource_registry = Arc::new(ResourceRegistry::new());

    let file = File::open(pipeline_file).unwrap();
    let reader = BufReader::new(&file);
    let mut archive = ZipArchive::new(reader).expect("zip");

    info!("File count: {}", archive.len());

    let mut json_file_path = None;

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
            info!("Found {:?}, extracting", filename);

            let json_file_dest = temp_target_dir.path().join(filename.clone());
            let mut outfile = fs::File::create(&json_file_dest).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();

            json_file_path = Some(json_file_dest);
        } else {
            let full_file_path = temp_target_dir.path().join(filename.to_owned());

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

    let json_file_path = if json_file_path.is_none() {
        error!("No .json file found");
        return Err(FileLoadError::NoJsonFile);
    } else {
        json_file_path.unwrap()
    };

    Ok((
        create_pipeline(&json_file_path),
        resource_registry,
        temp_target_dir,
    ))
}

fn create_pipeline(json_file: &Path) -> Pipeline {
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
