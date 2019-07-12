#![allow(dead_code)]

use std::ffi::CStr;

use capnp::{message::ReaderOptions, serialize};
use divvun_schema::{
    capnp_message,
    interface::{self, ModuleRunParameters, ModuleInterface},
    module_metadata,
    string_capnp::string,
    util,
};
use lazy_static::lazy_static;
use std::io::Cursor;

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const ModuleInterface) -> bool {
    interface::initialize(interface)
}

#[no_mangle]
pub extern "C" fn pipeline_run(p: *const ModuleRunParameters) -> bool {
    let p = unsafe { &*p };
    println!("hello from concat");

    let command = unsafe { CStr::from_ptr(p.command) }.to_string_lossy();
    let input_sizes = unsafe { std::slice::from_raw_parts(p.input_sizes, p.input_count) };
    let input = unsafe { std::slice::from_raw_parts(p.input, p.input_count) };

    println!("command = {}, input_count = {}", command, p.input_count);

    match &*command {
        "concat" => {
            let mut long_string = String::new();
            for i in 0..p.input_count {
                println!("i: {:?}", i);
                let slice = unsafe { std::slice::from_raw_parts(input[i], input_sizes[i]) };
                let mut cursor = Cursor::new(slice);
                let message = serialize::read_message(&mut cursor, ReaderOptions::new()).unwrap();
                let string = message.get_root::<string::Reader>().unwrap();
                let result = string.get_string().unwrap();

                println!("result: {:?}", result);

                long_string.push_str(result);
            }

            util::output_message(
                p.output,
                p.output_size,
                capnp_message!(string::Builder, builder => {
                    builder.set_string(&long_string);
                }),
            )
            .unwrap();

            true
        }
        _ => {
            util::output_message(
                p.output,
                p.output_size,
                divvun_schema::capnp_error!(
                    divvun_schema::error_capnp::pipeline_error::ErrorKind::UnknownCommand,
                    &format!("unknown command {}", command)
                ),
            )
            .unwrap();
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn pipeline_info(metadata: *mut *const u8, metadata_size: *mut usize) -> bool {
    lazy_static! {
        static ref MESSAGE: Vec<u8> = divvun_schema::util::message_to_vec(module_metadata! {
            name: "concat-string",
            version: "0.0.1",
            commands: {
                "concat" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
            }
        }).unwrap();
    }

    unsafe {
        *metadata = MESSAGE.as_ptr();
        *metadata_size = MESSAGE.len();
    }

    true
}
