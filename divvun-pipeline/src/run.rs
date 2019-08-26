use crate::{
    module::{AllocationType, ModuleAllocator, ModuleRegistry},
    pipeline::{Pipeline, PipelineData},
    resources::ResourceRegistry,
};
use capnp::{message::ReaderOptions, serialize};
use divvun_schema::string_capnp::string;
use log::info;
use std::{
    io::{Cursor, Read},
    path::PathBuf,
    sync::Arc,
};

const DEFAULT_MODULE_SEARCH_PATH: &str = "modules";

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct PipelineRunConfiguration {
    #[builder(default = PathBuf::from(DEFAULT_MODULE_SEARCH_PATH))]
    module_search_path: PathBuf,
    pipeline: Pipeline,
    resources: Arc<ResourceRegistry>,
    input: Vec<u8>,
    #[builder(default = "AllocationType::Memory")]
    allocation_type: AllocationType,
}

pub struct PipelineRunOutput {
    allocator: Arc<ModuleAllocator>,
    pub output: Box<dyn Read>,
}

impl PipelineRunConfiguration {
    pub async fn run(&self) -> PipelineRunOutput {
        let allocator = Arc::new(ModuleAllocator::new(self.allocation_type));
        let mut registry =
            ModuleRegistry::new(Arc::clone(&allocator), Arc::clone(&self.resources)).unwrap();
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
        info!("output size {}", output_size);
        let cursor = Cursor::new(slice);

        PipelineRunOutput {
            allocator,
            output: Box::new(cursor),
        }
    }
}

pub async fn run(
    pipeline: Pipeline,
    resources: Arc<ResourceRegistry>,
    input: Vec<u8>,
) -> PipelineRunOutput {
    let pipeline = PipelineRunConfigurationBuilder::default()
        .pipeline(pipeline)
        .resources(resources)
        .input(input)
        .build()
        .unwrap();
    pipeline.run().await
}
