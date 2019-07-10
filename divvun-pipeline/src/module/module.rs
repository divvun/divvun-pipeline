use capnp::message::{HeapAllocator, Reader, TypedReader};
use divvun_schema::error_capnp::pipeline_error;
use divvun_schema::interface::PipelineInterface;
use std::ffi::CStr;
use std::fmt;

use log::{error, info};
use parking_lot::Mutex;
use std::error::Error;
use std::ffi::CString;
use std::io::Cursor;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use super::ModuleAllocator;
use crate::resources::{ResourceHandle, ResourceRegistry};

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

struct ModuleInterfaceData {
    pub allocator: Arc<ModuleAllocator>,
    pub resource_registry: Arc<ResourceRegistry>,
    resource_handles: Mutex<Vec<Arc<ResourceHandle>>>,
}

impl ModuleInterfaceData {
    pub fn new(
        allocator: Arc<ModuleAllocator>,
        resource_registry: Arc<ResourceRegistry>,
    ) -> ModuleInterfaceData {
        ModuleInterfaceData {
            allocator,
            resource_registry,
            resource_handles: Mutex::new(Vec::new()),
        }
    }

    pub fn load_resource(&self, name: &str) -> Option<Arc<ResourceHandle>> {
        if let Some(handle) = self.resource_registry.get(name) {
            let handle = Arc::new(handle);
            // Keep track of the handle
            self.resource_handles.lock().push(handle.clone());
            return Some(handle);
        }

        None
    }
}

// Actual C interface
extern "C" fn alloc(data: *mut c_void, size: usize) -> *mut u8 {
    let data = data as *mut ModuleInterfaceData;
    unsafe {
        println!("ALLOC ON RUST: {:?}", data);
        println!(
            "ALLOC ON RUST allocator: {:?}",
            &*(*data).allocator as *const _
        );
    }
    // let data = unsafe { (*data).lock() };
    unsafe {
        (*data)
            .allocator
            .alloc(size)
            .unwrap_or(std::ptr::null_mut())
    }
}

extern "C" fn load_resource(
    data: *mut c_void,
    name: *const c_char,
    output: *mut *const u8,
    output_size: *mut usize,
) -> bool {
    let name = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let data = data as *mut ModuleInterfaceData;
    // let mut data = unsafe { (*data).lock() };

    let handle = unsafe { (*data).load_resource(&*name) };

    if let Some(handle) = handle {
        unsafe {
            *output = handle.as_ptr().unwrap();
            *output_size = handle.size().unwrap();
        }
        return true;
    }

    false
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
    interface: Arc<PipelineInterface>,
    interface_data: Arc<ModuleInterfaceData>,
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
        resource_registry: Arc<ResourceRegistry>,
        file_name: &Path,
    ) -> Result<Arc<Module>, Box<dyn Error>> {
        let lib = libloading::Library::new(file_name)?;

        println!("allocator A {:?}", &*allocator as *const _);
        let interface_data = Arc::new(ModuleInterfaceData::new(
            allocator.clone(),
            resource_registry,
        ));

        println!("interface_data B {:?}", &*interface_data as *const _);
        println!("allocator B {:?}", &*interface_data.allocator as *const _);

        let interface = Arc::new(PipelineInterface {
            data: &*interface_data as *const _ as *mut _,
            alloc_fn: alloc,
            load_resource_fn: load_resource,
        });

        println!("interface_data C {:?}", interface.data);
        println!("allocator C {:?}", unsafe {
            &*(*(interface.data as *const ModuleInterfaceData)).allocator as *const _
        });

        println!("if load {:?}", interface);

        let module = Arc::new(Module {
            library: lib,
            allocator,
            interface,
            interface_data,
            metadata: None,
        });

        module.call_init()?;

        // let metadata = module.call_info()?;
        // log_metadata(&metadata)?;

        // module.metadata = Some(Mutex::new(metadata));

        Ok(module)
    }

    /// The module's metadata
    pub fn metadata(&self) -> &Option<Mutex<MetadataType>> {
        &self.metadata
    }

    fn call_init(&self) -> Result<(), Box<dyn Error>> {
        let func: libloading::Symbol<PipelinInitFn> =
            unsafe { self.library.get(b"pipeline_init")? };

        info!("pipline_init");
        info!("init ptr {:?}", &*self.interface as *const _);
        let result = func(&*self.interface);
        info!("pipeline_init result: {}", result);

        if !result {
            return Err(PipelineRunError::InitializeFailed.into());
        }

        Ok(())
    }

    fn call_info(&self) -> Result<MetadataType, Box<dyn Error>> {
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
        let msg = divvun_schema::util::read_message::<
            divvun_schema::module_metadata_capnp::module_metadata::Owned,
        >(metadata, metadata_size)?;

        Ok(msg)
    }

    pub fn call_run(
        &self,
        command: &str,
        input: Vec<*const u8>,
        input_sizes: Vec<usize>,
    ) -> Result<PipelineRunResult, Box<dyn Error>> {
        println!("if R A{:?}", self.interface);
        let func: libloading::Symbol<PipelineRunFn> = unsafe { self.library.get(b"pipeline_run")? };

        let command = CString::new(command)?;
        let mut output: *const u8 = std::ptr::null();
        let mut output_size: usize = 0;
        println!("if R B {:?}", self.interface);
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
            let msg =
                divvun_schema::util::read_message::<pipeline_error::Owned>(output, output_size)?;
            let message = msg.get()?.get_message()?;
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

impl Drop for Module {
    fn drop(&mut self) {
        println!("Module dropped");
    }
}
