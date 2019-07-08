use memmap::{Mmap, MmapOptions};

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

pub trait LoadableResource {
    fn load(&mut self) -> Result<(), Box<dyn Error>>;
    fn unload(&mut self) -> Result<(), Box<dyn Error>>;

    fn size(&self) -> Option<usize>;
    fn as_ptr(&self) -> Option<*const u8>;
}

pub struct FileResource {
    path: PathBuf,
    mmap: Option<Mmap>,
}

impl LoadableResource for FileResource {
    fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(self.path)?;
        self.mmap = Some(unsafe { MmapOptions::new().map(&file)? });
        Ok(())
    }

    fn unload(&mut self) -> Result<(), Box<dyn Error>> {
        let _ = self.mmap.take();
        Ok(())
    }

    fn size(&self) -> Options<usize> {
        self.mmap.map(|mmap| mmap.len())
    }

    fn as_ptr(&self) -> Options<*const u8> {
        self.mmap.map(|mmap| mmap.as_ptr())
    }
}

pub struct ResourceRegistry {
    available: HashMap<String, LoadableResource>,
}

impl ResourceRegistry {
    pub fn add_resource(&mut self, name: &str, resource: LoadableResource) {
        self.available.insert(name, resource);
    }
}
