#![allow(dead_code)]

use std::ffi::CStr;
use std::os::raw::c_char;

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
    match &*command {
        "badazzle" => {
            for i in 0..input_count {
                divvun_schema::util::output_string(
                    "a computatoion".to_string(),
                    output,
                    output_size,
                );
                return true;
            }

            false
        }
        "load_nude_tayne" => {
            for i in 0..input_count {
                divvun_schema::util::output_string(
                    "a picture of a handsome man".to_string(),
                    output,
                    output_size,
                );
                return true;
            }

            false
        }
        _ => {
            let out = format!("unknown command {}", command);

            divvun_schema::util::output_string(out, output, output_size);
            false
        }
    }
}

// // Pseudocode macro for declaring module metadata
// static metadata: ModuleMetadata = module_metadata! {
//   name: "reverse-string",
//   commands: {
//     "reverse" => { input: ReverseInput, output: TokenizedString }
//   }
// }

// struct ReverseInput {
//     string: InputString,
//     audio: InputAudio
// }

// extern "C" fn pipeline_info() -> *const ModuleMetadata {
//     metadata.as_ptr()
// }
