use std::{io::Cursor, path::Path, sync::Arc};

use crate::{
    module::{AllocationType, ModuleAllocator, ModuleRegistry},
    pipeline::{Pipeline, PipelineData},
};

use crate::resources::ResourceRegistry;
use divvun_schema::string_capnp::string;

use capnp::{message::ReaderOptions, serialize};

pub async fn run(pipeline: &Pipeline, input: Vec<u8>) -> String {
    let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
    let resources = Arc::new(ResourceRegistry::new());
    let mut registry = ModuleRegistry::new(allocator, resources).unwrap();
    registry.add_search_path(Path::new("../modules"));
    let registry = Arc::new(registry);

    let result = pipeline
        .run(
            registry.clone(),
            Arc::new(vec![Arc::new(PipelineData {
                data: input.as_ptr(),
                size: input.len(),
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
