mod bindings;
use std::{ffi::c_void, os::raw::c_char};

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
