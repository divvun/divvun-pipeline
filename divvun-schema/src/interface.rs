use capnp::message::TypedReader;
use std::fmt;
use std::os::raw::c_void;

pub type AllocFn = extern "C" fn(*mut c_void, usize) -> *mut u8;

#[derive(Debug)]
#[repr(C)]
pub struct PipelineInterface {
    pub allocator: *mut c_void,
    pub alloc_fn: AllocFn,
}

impl Drop for PipelineInterface {
    fn drop(&mut self) {
        println!("drop");
    }
}
unsafe impl Send for PipelineInterface {}
unsafe impl Sync for PipelineInterface {}

impl PipelineInterface {
    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        println!("lloc {:?} {:?}", self.allocator, self.alloc_fn);
        let result = (self.alloc_fn)(self.allocator, size);
        if result == std::ptr::null_mut() {
            return None;
        }
        Some(result)
    }
}

pub static mut PIPELINE_INTERFACE: Option<*const PipelineInterface> = None;

/// To be called by the pipeline module to allocate memory needed for large chunks of data
pub fn allocate(size: usize) -> Option<*mut u8> {
    unsafe {
        if let Some(interface) = PIPELINE_INTERFACE {
            println!(
                "interface {:?} {:?} size {}",
                std::thread::current().id(),
                interface,
                size
            );
            (*interface).alloc(size)
        } else {
            None
        }
    }
}

/// To be called by the pipeline module's pipeline_init function to initialize the SDK
pub fn initialize(interface: *const PipelineInterface) -> bool {
    unsafe {
        println!(
            "initialize {:?} {:?} {:?} {:?}",
            std::thread::current().id(),
            interface,
            (*interface).allocator,
            (*interface).alloc_fn
        );
        PIPELINE_INTERFACE = Some(interface);
    }
    true
}
