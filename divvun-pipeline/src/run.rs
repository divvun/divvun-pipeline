use std::{io::Cursor, path::PathBuf, sync::Arc};

use crate::{
    module::{AllocationType, ModuleAllocator, ModuleRegistry},
    pipeline::{Pipeline, PipelineData},
};

use crate::resources::ResourceRegistry;
use divvun_schema::string_capnp::string;

use capnp::{message::ReaderOptions, serialize};

const DEFAULT_MODULE_SEARCH_PATH: &str = "modules";

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct PipelinRunConfiguration {
    #[builder(default = "PathBuf::from(DEFAULT_MODULE_SEARCH_PATH)")]
    module_search_path: PathBuf,
    pipeline: Pipeline,
    resources: Arc<ResourceRegistry>,
    input: Vec<u8>,
}

impl PipelinRunConfiguration {
    pub async fn run(&self) -> String {
        let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
        let mut registry = ModuleRegistry::new(allocator, Arc::clone(&self.resources)).unwrap();
        registry.add_search_path(&self.module_search_path);
        let registry = Arc::new(registry);

        let result = self
            .pipeline
            .run(
                registry.clone(),
                Arc::new(vec![Arc::new(PipelineData {
                    data: self.input.as_ptr(),
                    size: self.input.len(),
                })]),
            )
            .await;

        let inter_output = result.unwrap();
        let output = inter_output.get(0).unwrap();

        let output_data = output.data;
        let output_size = output.size;

        let slice = unsafe { std::slice::from_raw_parts(output_data, output_size) };

        let mut cursor = Cursor::new(slice);
        let message = serialize::read_message(&mut cursor, ReaderOptions::new()).unwrap();
        let string = message.get_root::<string::Reader>().unwrap();
        let result = string.get_string().unwrap();

        result.to_owned()
    }
}

pub async fn run(pipeline: Pipeline, resources: Arc<ResourceRegistry>, input: Vec<u8>) -> String {
    let pipeline = PipelinRunConfigurationBuilder::default()
        .pipeline(pipeline)
        .resources(resources)
        .input(input)
        .build()
        .unwrap();
    pipeline.run().await
}
