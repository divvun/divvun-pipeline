use capnp::message::{HeapAllocator, Reader, TypedReader};
use divvun_schema::error_capnp::pipeline_error;
use divvun_schema::interface::PipelineInterface;
use std::fmt;

use log::{error, info};
use std::error::Error;
use std::ffi::CString;
use std::io::Cursor;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use super::ModuleAllocator;

type PipelineRunFn = fn(
    command: *const c_char,
    input_count: usize,
    input: *const *const u8,
    input_sizes: *const usize,
    output: *mut *const u8,
    output_size: *mut usize,
) -> bool;

type PipelinInitFn = fn(*const PipelineInterface) -> bool;
type PipelinInfoFn = fn(*mut *const u8, *mut usize) -> bool;

extern "C" fn alloc(allocator: *mut c_void, size: usize) -> *mut u8 {
    let allocator = allocator as *mut ModuleAllocator;
    unsafe { (*allocator).alloc(size).unwrap_or(std::ptr::null_mut()) }
}

pub type MetadataType = TypedReader<
    capnp::serialize::OwnedSegments,
    divvun_schema::module_metadata_capnp::module_metadata::Owned,
>;
pub type PipelineErrorType = TypedReader<capnp::serialize::OwnedSegments, pipeline_error::Owned>;

#[derive(Debug)]
pub struct PipelineRunResult {
    pub output: *const u8,
    pub output_size: usize,
}

pub enum PipelineRunError {
    Error(PipelineErrorType),
    InitializeFailed,
    InfoFailed,
}

impl PipelineRunError {
    pub fn pipeline_error(&self) -> Option<&PipelineErrorType> {
        match self {
            PipelineRunError::Error(ref error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Debug for PipelineRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipelineRunError::Error(ref error_struct) => {
                write!(f, "Pipeline failed to run: ")?;
                writeln!(
                    f,
                    "{}",
                    error_struct
                        .get()
                        .and_then(|e| e.get_message())
                        .unwrap_or("failed to get error")
                )?;
            }
            e => write!(f, "{:?}", e)?,
        };

        Ok(())
    }
}

impl fmt::Display for PipelineRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipelineRunError::Error(ref error_struct) => {
                writeln!(f, "Pipeline failed to run")?;
                writeln!(f, "{:?}", error_struct.get().and_then(|e| e.get_message()))?;
            }
            e => write!(f, "{}", e)?,
        };

        Ok(())
    }
}

impl Error for PipelineRunError {}

pub struct Module {
    library: libloading::Library,
    allocator: Arc<ModuleAllocator>,
    interface: PipelineInterface,
    metadata: Option<Mutex<MetadataType>>,
}

fn log_metadata(metadata: &MetadataType) -> Result<(), Box<dyn Error>> {
    let metadata_inner = metadata.get()?;
    let module_name = metadata_inner.get_module_name()?;
    let commands = metadata_inner.get_commands()?;
    info!("ModuleMetadata {{");
    info!("  moduleName: \"{}\",", module_name);
    info!("  commands: {{");
    for i in 0..commands.len() {
        let command = commands.get(i);
        let inputs = command.get_inputs()?;
        let command_name = command.get_name()?;
        info!("    {} => {{", command_name);
        info!("      output: {},", command.get_output());
        info!(
            "      inputs: [{}],",
            (0..inputs.len())
                .map(|i| inputs.get(i).to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        info!("    }}");
    }
    info!("  }}");
    info!("}}");
    Ok(())
}

impl Module {
    /// Load, initialize and request metadata of the module
    pub fn load(
        allocator: Arc<ModuleAllocator>,
        file_name: &Path,
    ) -> Result<Module, Box<dyn Error>> {
        let lib = libloading::Library::new(file_name)?;
        let interface = PipelineInterface {
            allocator: &*allocator as *const _ as *mut c_void,
            alloc_fn: alloc,
        };

        let mut module = Module {
            library: lib,
            allocator,
            interface,
            metadata: None,
        };

        module.call_init()?;

        let metadata = module.call_info()?;
        log_metadata(&metadata)?;

        module.metadata = Some(Mutex::new(metadata));

        Ok(module)
    }

    /// The module's metadata
    pub fn metadata(&self) -> &Option<Mutex<MetadataType>> {
        &self.metadata
    }

    pub fn call_init(&mut self) -> Result<(), Box<dyn Error>> {
        let func: libloading::Symbol<PipelinInitFn> =
            unsafe { self.library.get(b"pipeline_init")? };

        info!("pipline_init");
        let result = func(&self.interface);
        info!("pipeline_init result: {}", result);

        if !result {
            return Err(PipelineRunError::InitializeFailed.into());
        }

        Ok(())
    }

    pub fn call_info(&mut self) -> Result<MetadataType, Box<dyn Error>> {
        let func: libloading::Symbol<PipelinInfoFn> =
            unsafe { self.library.get(b"pipeline_info")? };

        let mut metadata: *const u8 = std::ptr::null_mut();
        let mut metadata_size: usize = 0;

        info!("pipeline_info");
        let result = func(&mut metadata, &mut metadata_size);
        info!("pipeline_info result: {}", result);
        if !result {
            return Err(PipelineRunError::InfoFailed.into());
        }
        let slice = unsafe { std::slice::from_raw_parts(metadata, metadata_size) };
        let mut cursor = Cursor::new(slice);

        let message =
            capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new())
                .unwrap();

        Ok(TypedReader::new(message))
    }

    pub fn call_run(
        &mut self,
        command: &str,
        input: Vec<*const u8>,
        input_sizes: Vec<usize>,
    ) -> Result<PipelineRunResult, Box<dyn Error>> {
        let func: libloading::Symbol<PipelineRunFn> = unsafe { self.library.get(b"pipeline_run")? };

        let command = CString::new(command)?;
        let mut output: *const u8 = std::ptr::null();
        let mut output_size: usize = 0;
        let result = func(
            command.as_ptr(),
            input.len(),
            input.as_ptr(),
            input_sizes.as_ptr(),
            &mut output,
            &mut output_size,
        );

        info!("result = {} output size = {}", result, output_size);
        if !result {
            let output_slice = unsafe { std::slice::from_raw_parts(output, output_size) };
            let mut output_cursor = Cursor::new(output_slice);
            let msg = ::capnp::serialize::read_message(
                &mut output_cursor,
                capnp::message::ReaderOptions::new(),
            )?;

            let error = msg.get_root::<pipeline_error::Reader>()?;
            let message = error.get_message()?;
            error!("an error happened: {}", message);
            Err(Box::new(PipelineRunError::Error(TypedReader::from(msg))))
        } else {
            Ok(PipelineRunResult {
                output,
                output_size,
            })
        }
    }
}
