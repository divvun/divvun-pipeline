#![allow(dead_code)]

use std::ffi::CStr;
use std::os::raw::c_char;

use capnp::message::ReaderOptions;
use capnp::serialize;
use divvun_schema::capnp_message;
use divvun_schema::interface::{self, PipelineInterface};
use divvun_schema::string_capnp::string;
use divvun_schema::util;
use lazy_static::lazy_static;
use std::io::Cursor;

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const PipelineInterface) -> bool {
    interface::initialize(interface)
}

#[no_mangle]
extern "C" fn pipeline_run(
    command: *const c_char,
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
        "concat" => {
            let mut long_string = String::new();
            for i in 0..input_count {
                let slice = unsafe { std::slice::from_raw_parts(input[i], input_sizes[i]) };
                let mut cursor = Cursor::new(slice);
                let message = serialize::read_message(&mut cursor, ReaderOptions::new()).unwrap();
                let string = message.get_root::<string::Reader>().unwrap();
                let result = string.get_string().unwrap();

                long_string.push_str(result);
            }

            util::output_message(
                output,
                output_size,
                capnp_message!(string::Builder, builder => {
                    builder.set_string(&long_string);
                }),
            )
            .unwrap();

            true
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
        static ref MESSAGE: Vec<u8> = divvun_schema::util::message_to_vec(
            capnp_message!(divvun_schema::module_metadata_capnp::module_metadata::Builder, builder => {
                builder.set_module_name("do-things-strings");
                let mut commands = builder.init_commands(1);
                {
                    use capnp::traits::HasTypeId;
                    let mut command = commands.reborrow().get(0);
                    command.set_name("concat");
                    command.set_output(divvun_schema::string_capnp::string::Builder::type_id());
                    let inputs = command.init_inputs(1);
                    {
                        inputs.reborrow().set(0, divvun_schema::string_capnp::string::Builder::type_id());
                    }
                }
            }),
        ).unwrap();
    }

    unsafe {
        *metadata = MESSAGE.as_ptr();
        *metadata_size = MESSAGE.len();
    }

    true
}
