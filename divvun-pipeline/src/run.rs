use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use std::path::Path;
use std::env;

use log::info;
use clap::{App, Arg, crate_version};
use serde_json::Value;

use crate::module::{AllocationType, ModuleAllocator, ModuleRegistry};
use crate::pipeline::{Pipeline, PipelineData};

use divvun_schema::capnp_message;
use divvun_schema::string_capnp::string;

use capnp::message::ReaderOptions;
use capnp::serialize;

pub async fn run(pipeline: &Pipeline) -> String {
    let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
    let mut registry = ModuleRegistry::new(allocator).unwrap();
    registry.add_search_path(Path::new("../modules"));
    let registry = Arc::new(registry);

    let msg = capnp_message!(string::Builder, builder => {
        builder.set_string("Hello world!");
    });

    let msg_vec = divvun_schema::util::message_to_vec(msg).unwrap();

    let result = pipeline
        .run(
            registry.clone(),
            Arc::new(vec![Arc::new(PipelineData {
                data: msg_vec.as_ptr(),
                size: msg_vec.len(),
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