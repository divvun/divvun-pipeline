use capnp::{message::ReaderOptions, serialize};
use divvun_schema::{
    capnp_message,
    interface::{self, ModuleRunParameters, PipelineInterface},
    module_metadata,
    string_capnp::string,
    util,
};
use lazy_static::lazy_static;
use log::info;
use std::{ffi::CStr, io::Cursor, os::raw::c_char, str};
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
    let command = p.command();

    let input_sizes = p.input_sizes();
    let input = p.input();

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
            for i in 0..input.len() {
                let message =
                    util::read_message::<string::Owned>(p.get_input(i), p.get_input_size(i))
                        .unwrap();
                let input_data = message.get().unwrap().get_string().unwrap();

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

                let pmatch_resource = p.get_parameter(0);
                let pmatch = interface::load_resource(&*pmatch_resource)
                    .expect("pmatch resource doesn't exist");

                let mut output_size: usize = 0;
                // hfst_run runs the hfst tokenizer and writes into a std::stringstream
                // If we want to avoid the copying here, we have to make a custom STL allocator in C++
                // to use our allocator, and then get a pointer to that buffer back instead :)
                let stream = unsafe {
                    hfst_run(
                        &settings,
                        pmatch.as_ptr(),
                        pmatch.size(),
                        input_data.as_ptr(),
                        input_data.len(),
                        &mut output_size,
                    )
                };

                // When we have a interface deallocate function we can use our allocation system to
                // allocate the temporary buffer.
                // let mut output = interface::allocate(output_size).expect("failed to allocate");
                let mut output = vec![0u8; output_size];
                unsafe {
                    hfst_copy_output(stream, output.as_mut_ptr(), output_size);
                }

                unsafe {
                    hfst_free(stream);
                }

                util::output_message(
                    p.output,
                    p.output_size,
                    capnp_message!(string::Builder, builder => {
                        // This copies the string from our buffer into a buffer allocated by capnp
                        // Ideally we override the capnp allocator to use our allocator
                        // and at the same time use hfst_copy_output to write directly into
                        // our allocated buffer.
                        // This /should/ be possible through capnp's internal functions
                        // init_text_pointer / set_text_pointer are the relevant functions
                        builder.set_string(unsafe { str::from_utf8_unchecked(std::slice::from_raw_parts(output.as_ptr(), output_size))});
                    })
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
