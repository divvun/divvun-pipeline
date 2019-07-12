use capnp::message::TypedReader;
use divvun_schema::{
    error_capnp::pipeline_error,
    interface::{ModuleRunParameters, ModuleInterface},
};
use std::{ffi::CStr, fmt};

use log::{error, info};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    error::Error,
    ffi::CString,
    os::raw::{c_char, c_void},
    path::Path,
    sync::Arc,
};

use super::ModuleAllocator;
use crate::resources::{ResourceHandle, ResourceRegistry};

type ModuleRunFn = fn(*const ModuleRunParameters) -> bool;

type ModuleInitFn = fn(*const ModuleInterface) -> bool;
type ModuleInfoFn = fn(*mut *const u8, *mut usize) -> bool;

struct ModuleInterfaceData {
    pub allocator: Arc<ModuleAllocator>,
    pub resource_registry: Arc<ResourceRegistry>,
    resource_handles: Mutex<HashMap<String, Arc<ResourceHandle>>>,
}

impl ModuleInterfaceData {
    pub fn new(
        allocator: Arc<ModuleAllocator>,
        resource_registry: Arc<ResourceRegistry>,
    ) -> ModuleInterfaceData {
        ModuleInterfaceData {
            allocator,
            resource_registry,
            resource_handles: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_resource(&self, name: &str) -> Option<Arc<ResourceHandle>> {
        if let Some(handle) = self.resource_registry.get(name) {
            let handle = Arc::new(handle);
            // Keep track of the handle
            self.resource_handles
                .lock()
                .insert(name.to_string(), handle.clone());
            return Some(handle);
        }

        None
    }

    pub fn release_resource(&self, name: &str) -> bool {
        return self.resource_handles.lock().remove(name).is_some();
    }
}

// Actual C interface
extern "C" fn alloc(data: *mut c_void, size: usize) -> *mut u8 {
    let data = data as *mut ModuleInterfaceData;

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

extern "C" fn release_resource(data: *mut c_void, name: *const c_char) -> bool {
    let name = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let data = data as *mut ModuleInterfaceData;

    unsafe { (*data).release_resource(&*name) }
}

pub type MetadataType = TypedReader<
    capnp::serialize::OwnedSegments,
    divvun_schema::module_metadata_capnp::module_metadata::Owned,
>;
pub type ModuleErrorType = TypedReader<capnp::serialize::OwnedSegments, pipeline_error::Owned>;

#[derive(Debug)]
pub struct ModuleRunResult {
    pub output: *const u8,
    pub output_size: usize,
}

pub enum ModuleRunError {
    Error(ModuleErrorType),
    InitializeFailed,
    InfoFailed,
}

impl ModuleRunError {
    pub fn pipeline_error(&self) -> Option<&ModuleErrorType> {
        match self {
            ModuleRunError::Error(ref error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Debug for ModuleRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModuleRunError::Error(ref error_struct) => {
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

impl fmt::Display for ModuleRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModuleRunError::Error(ref error_struct) => {
                writeln!(f, "Pipeline failed to run")?;
                writeln!(f, "{:?}", error_struct.get().and_then(|e| e.get_message()))?;
            }
            e => write!(f, "{}", e)?,
        };

        Ok(())
    }
}

impl Error for ModuleRunError {}

#[allow(unused)]
pub struct Module {
    library: libloading::Library,
    allocator: Arc<ModuleAllocator>,
    interface: Arc<ModuleInterface>,
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

        let interface_data = Arc::new(ModuleInterfaceData::new(
            allocator.clone(),
            resource_registry,
        ));

        let interface = Arc::new(ModuleInterface {
            data: &*interface_data as *const _ as *mut _,
            alloc_fn: alloc,
            load_resource_fn: load_resource,
            release_resource_fn: release_resource,
        });

        let mut module = Module {
            library: lib,
            allocator,
            interface,
            interface_data,
            metadata: None,
        };

        module.call_init()?;

        let metadata = module.call_info()?;
        log_metadata(&metadata)?;

        module.metadata = Some(Mutex::new(metadata));

        Ok(Arc::new(module))
    }

    /// The module's metadata
    pub fn metadata(&self) -> &Option<Mutex<MetadataType>> {
        &self.metadata
    }

    fn call_init(&self) -> Result<(), Box<dyn Error>> {
        let func: libloading::Symbol<ModuleInitFn> =
            unsafe { self.library.get(b"pipeline_init")? };

        info!("pipline_init");
        let result = func(&*self.interface);
        info!("pipeline_init result: {}", result);

        if !result {
            return Err(ModuleRunError::InitializeFailed.into());
        }

        Ok(())
    }

    fn call_info(&self) -> Result<MetadataType, Box<dyn Error>> {
        let func: libloading::Symbol<ModuleInfoFn> =
            unsafe { self.library.get(b"pipeline_info")? };

        let mut metadata: *const u8 = std::ptr::null_mut();
        let mut metadata_size: usize = 0;

        info!("pipeline_info");
        let result = func(&mut metadata, &mut metadata_size);
        info!("pipeline_info result: {}", result);
        if !result {
            return Err(ModuleRunError::InfoFailed.into());
        }
        let msg = divvun_schema::util::read_message::<
            divvun_schema::module_metadata_capnp::module_metadata::Owned,
        >(metadata, metadata_size)?;

        Ok(msg)
    }

    pub fn call_run(
        &self,
        command: &str,
        parameters: Option<&Vec<String>>,
        input: Vec<*const u8>,
        input_sizes: Vec<usize>,
    ) -> Result<ModuleRunResult, Box<dyn Error>> {
        let func: libloading::Symbol<ModuleRunFn> = unsafe { self.library.get(b"pipeline_run")? };

        let command = CString::new(command)?;
        let mut output: *const u8 = std::ptr::null();
        let mut output_size: usize = 0;

        let parameters = parameters
            .map(|parameters| {
                parameters
                    .iter()
                    .map(|p| CString::new(&**p).expect("valid parameter"))
                    .collect::<Vec<CString>>()
            })
            .unwrap_or_else(|| vec![]);;

        let parameter_ptr: Vec<*const c_char> =
            parameters.iter().map(|p| p.as_ptr()).collect::<Vec<_>>();

        let parameters = ModuleRunParameters {
            command: command.as_ptr(),
            parameters: parameter_ptr.as_ptr(),
            parameter_count: parameter_ptr.len(),
            input: input.as_ptr(),
            input_count: input.len(),
            input_sizes: input_sizes.as_ptr(),
            output: &mut output,
            output_size: &mut output_size,
        };

        let result = func(&parameters);

        info!("result = {} output size = {}", result, output_size);
        if !result {
            let msg =
                divvun_schema::util::read_message::<pipeline_error::Owned>(output, output_size)?;
            let message = msg.get()?.get_message()?;
            error!("an error happened: {}", message);
            Err(Box::new(ModuleRunError::Error(TypedReader::from(msg))))
        } else {
            Ok(ModuleRunResult {
                output,
                output_size,
            })
        }
    }
}
