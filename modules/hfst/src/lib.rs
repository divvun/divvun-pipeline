use capnp::{message::ReaderOptions, serialize};
use divvun_schema::{
    capnp_message,
    interface::{self, ModuleRunParameters, PipelineInterface},
    module_metadata,
    string_capnp::string,
    util,
};
use lazy_static::lazy_static;
use std::{ffi::CStr, io::Cursor, os::raw::c_char};
mod bindings;
use std::ffi::c_void;

extern "C" {
    fn hfst_run(
        settings: *const bindings::hfst_ol_tokenize_TokenizeSettings,
        pmatch_data: *const u8,
        pmatch_size: usize,
        input_data: *const u8,
        input_size: usize,
        output_size: *mut usize,
    ) -> *const c_void;

    fn hfst_free(stream: *const c_void);
    fn hfst_copy_output(stream: *const c_void, output: *mut u8, size: usize);
}

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const PipelineInterface) -> bool {
    interface::initialize(interface)
}

#[no_mangle]
pub extern "C" fn pipeline_run(p: *const ModuleRunParameters) -> bool {
    let p = unsafe { &*p };
    let command = unsafe { CStr::from_ptr(p.command) }.to_string_lossy();

    let input_sizes = unsafe { std::slice::from_raw_parts(p.input_sizes, p.input_count) };
    let input = unsafe { std::slice::from_raw_parts(p.input, p.input_count) };

    if input.len() == 0 {
        util::output_message(
            p.output,
            p.output_size,
            divvun_schema::capnp_error!(
                divvun_schema::error_capnp::pipeline_error::ErrorKind::ModuleError,
                "no input provided"
            ),
        )
        .unwrap();
        return false;
    }

    match &*command {
        "tokenize" => {
            for i in 0..p.input_count {
                let message =
                    util::read_message::<string::Owned>(input[i], input_sizes[i]).unwrap();
                let string = message.get().unwrap();
                let result: String = string.get_string().unwrap().chars().rev().collect();

                // do hfst tokenize
                let settings = bindings::hfst_ol_tokenize_TokenizeSettings {
                    output_format: bindings::hfst_ol_tokenize_OutputFormat_giellacg,
                    tokenize_multichar: false,
                    print_weights: true,
                    print_all: true,
                    dedupe: true,
                    max_weight_classes: std::os::raw::c_int::max_value(),
                    // Defaults
                    beam: -1.0,
                    time_cutoff: 0.0,
                    verbose: true,
                    weight_cutoff: -1.0,
                };

                // println!("hfst input: {}", String::from_utf8_lossy(input_data));
                // let mut output_size: usize = 0;
                // let stream = unsafe {
                //     hfst_run(
                //         &settings,
                //         pmatch.as_ptr(),
                //         pmatch.len(),
                //         input_data.as_ptr(),
                //         input_data.len(),
                //         &mut output_size,
                //     )
                // };

                // println!("output size: {}", output_size);

                // let mut output = vec![0u8; output_size];
                // unsafe {
                //     hfst_copy_output(stream, output.as_mut_ptr(), output.len());
                // }

                // let output = String::from_utf8_lossy(&output);
                // println!("hfst output: {}", output);

                // unsafe {
                //     hfst_free(stream);
                // }

                // util::output_message(
                //     output,
                //     output_size,
                //     capnp_message!(string::Builder, builder => {
                //         builder.set_string(&result);
                //     }),
                // )
                // .unwrap();

                println!("returning from reverse");

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
            name: "hfst",
            version: "0.0.1",
            commands: {
                "tokenize" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
                // "reverse_resource" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
            }
        }).unwrap();
    }

    unsafe {
        *metadata = MESSAGE.as_ptr();
        *metadata_size = MESSAGE.len();
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let settings = bindings::hfst_ol_tokenize_TokenizeSettings {
            output_format: bindings::hfst_ol_tokenize_OutputFormat_giellacg,
            tokenize_multichar: false,
            print_weights: true,
            print_all: true,
            dedupe: true,
            max_weight_classes: std::os::raw::c_int::max_value(),
            // Defaults
            beam: -1.0,
            time_cutoff: 0.0,
            verbose: true,
            weight_cutoff: -1.0,
        };

        let pmatch = std::fs::read("../../se_zcheck/tokeniser-gramcheck-gt-desc.pmhfst").unwrap();
        let input_data = b"Hello world please correc this or something";
        println!("hfst input: {}", String::from_utf8_lossy(input_data));
        let mut output_size: usize = 0;
        let stream = unsafe {
            hfst_run(
                &settings,
                pmatch.as_ptr(),
                pmatch.len(),
                input_data.as_ptr(),
                input_data.len(),
                &mut output_size,
            )
        };

        println!("output size: {}", output_size);

        let mut output = vec![0u8; output_size];
        unsafe {
            hfst_copy_output(stream, output.as_mut_ptr(), output.len());
        }

        let output = String::from_utf8_lossy(&output);
        println!("hfst output: {}", output);

        unsafe {
            hfst_free(stream);
        }
    }
}
