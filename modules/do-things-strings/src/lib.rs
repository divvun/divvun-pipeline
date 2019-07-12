#![allow(dead_code)]

use divvun_schema::{
    capnp_message,
    interface::{self, ModuleRunParameters, ModuleInterface},
    module_metadata,
    string_capnp::string,
    util,
};
use lazy_static::lazy_static;
use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const ModuleInterface) -> bool {
    interface::initialize(interface)
}

#[no_mangle]
pub extern "C" fn pipeline_run(p: *const ModuleRunParameters) -> bool {
    let p = unsafe { &*p };
    let command = unsafe { CStr::from_ptr(p.command) }.to_string_lossy();

    match &*command {
        "badazzle" => {
            for _ in 0..p.input_count {
                util::output_message(
                    p.output,
                    p.output_size,
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
            for _ in 0..p.input_count {
                util::output_message(
                    p.output,
                    p.output_size,
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
