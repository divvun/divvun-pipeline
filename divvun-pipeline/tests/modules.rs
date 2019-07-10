use std::{path::Path, sync::Arc};

use divvun_pipeline::{
    module::*,
    pipeline::*,
    resources::{LoadableResource, Resource, ResourceRegistry},
};
use divvun_schema::{string_capnp::string, util};

mod common;

#[test]
fn load_run_module_memory() {
    let (registry, allocator, ..) = common::setup_test_registry(AllocationType::Memory);

    let mut module = registry.get_module("reverse_string").unwrap();
    let inputs: Vec<*const u8> = Vec::new();
    let input_sizes: Vec<usize> = Vec::new();

    let result = module.call_run("reverse", inputs, input_sizes);
    assert!(result.is_err());

    assert_eq!(allocator.total_size(), 64);
}

#[test]
fn load_run_module_file() {
    let (registry, allocator, ..) = common::setup_test_registry(AllocationType::File);

    let mut module = registry.get_module("reverse_string").unwrap();
    let inputs: Vec<*const u8> = Vec::new();
    let input_sizes: Vec<usize> = Vec::new();

    let result = module.call_run("reverse", inputs, input_sizes);
    assert!(result.is_err());
    // println!(
    //     "{}",
    //     result.unwrap_err().pipeline_error().unwrap().get_kind()
    // );
    assert_eq!(allocator.total_size(), 64);
}

#[test]
fn load_run_input_reverse() {
    let (registry, allocator, ..) = common::setup_test_registry(AllocationType::Memory);

    let mut module = registry.get_module("reverse_string").unwrap();

    let text = util::message_to_vec(divvun_schema::capnp_message!(string::Builder, builder => {
        builder.set_string("hello");
    }))
    .unwrap();

    let inputs: Vec<*const u8> = vec![text.as_ptr()];
    let input_sizes: Vec<usize> = vec![text.len()];

    let result = module.call_run("reverse", inputs, input_sizes);
    assert!(result.is_ok());

    let result = result.unwrap();

    let slice = unsafe { std::slice::from_raw_parts(result.output, result.output_size) };
    let mut cursor = std::io::Cursor::new(slice);

    let message =
        capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new()).unwrap();
    let text = message
        .get_root::<divvun_schema::string_capnp::string::Reader>()
        .unwrap();

    assert_eq!(text.get_string().unwrap(), "olleh");
}

#[test]
fn load_run_input_reverse_resource() {
    let (registry, allocator, resources) = common::setup_test_registry(AllocationType::Memory);

    let my_data = "Hello".as_bytes();
    resources.add_resource(
        "lol",
        LoadableResource::from(Resource::Bytes(my_data.to_owned())),
    );

    let module = registry.get_module("reverse_string").unwrap();

    let text = util::message_to_vec(divvun_schema::capnp_message!(string::Builder, builder => {
        builder.set_string("lol");
    }))
    .unwrap();

    let inputs: Vec<*const u8> = vec![text.as_ptr()];
    let input_sizes: Vec<usize> = vec![text.len()];

    let result = module.call_run("reverse_resource", inputs, input_sizes);
    assert!(result.is_ok());

    let result = result.unwrap();

    let slice = unsafe { std::slice::from_raw_parts(result.output, result.output_size) };
    let mut cursor = std::io::Cursor::new(slice);

    let message =
        capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new()).unwrap();
    let text = message
        .get_root::<divvun_schema::string_capnp::string::Reader>()
        .unwrap();

    assert_eq!(text.get_string().unwrap(), "olleH");
}
