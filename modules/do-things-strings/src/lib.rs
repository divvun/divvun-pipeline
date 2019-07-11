#![allow(dead_code)]

use capnp::{message::ReaderOptions, serialize};
use divvun_schema::{
    capnp_message,
    interface::{self, PipelineInterface},
    module_metadata,
    string_capnp::string,
    util,
};
use lazy_static::lazy_static;
use std::{ffi::CStr, io::Cursor, os::raw::c_char};

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const PipelineInterface) -> bool {
    interface::initialize(interface)
}

#[no_mangle]
extern "C" fn pipeline_run(
    command: *const c_char,
    parameters: *const *const c_char,
    parameter_count: usize,
    input_count: usize,
    input: *const *const u8,
    input_sizes: *const usize,
    output: *mut *const u8,
    output_size: *mut usize,
) -> bool {
    let command = unsafe { CStr::from_ptr(command) }.to_string_lossy();
    let input_sizes = unsafe { std::slice::from_raw_parts(input_sizes, input_count) };
    let input = unsafe { std::slice::from_raw_parts(input, input_count) };

    match &*command {
        "badazzle" => {
            for i in 0..input_count {
                util::output_message(
                    output,
                    output_size,
                    capnp_message!(string::Builder, builder => {
                        builder.set_string("A BIG COMPUTATIONS DONE HERE");
                    }),
                )
                .unwrap();
                return true;
            }

            false
        }
        "stuff" => {
            for i in 0..input_count {
                util::output_message(
                    output,
                    output_size,
                    capnp_message!(string::Builder, builder => {
                        builder.set_string("Here is a computation stuff!");
                    }),
                )
                .unwrap();
                return true;
            }

            false
        }
        _ => {
            util::output_message(
                output,
                output_size,
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
            name: "do-things-strings",
            version: "0.0.1",
            commands: {
                "badazzle" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
                "stuff" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
            }
        }).unwrap();
    }

    unsafe {
        *metadata = MESSAGE.as_ptr();
        *metadata_size = MESSAGE.len();
    }

    true
}
