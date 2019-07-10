use log::info;
use memmap::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::error::Error;
use tempfile::tempfile;

pub struct ModuleAllocator {
    allocation_type: AllocationType,
    mmaps: RwLock<Vec<MmapMut>>,
}

#[derive(Debug)]
pub enum AllocationType {
    File,
    Memory,
}

impl ModuleAllocator {
    pub fn new_default() -> ModuleAllocator {
        Self::new(AllocationType::Memory)
    }

    pub fn new(allocation_type: AllocationType) -> ModuleAllocator {
        ModuleAllocator {
            allocation_type,
            mmaps: RwLock::new(Vec::new()),
        }
    }

    pub fn total_size(&self) -> usize {
        info!("total size");
        self.mmaps.read().iter().map(|m| m.len()).sum()
    }

    pub fn alloc(&self, size: usize) -> Result<*mut u8, Box<dyn Error>> {
        info!("allocating {} bytes as {:?}", size, self.allocation_type);
        let mut mmap = match self.allocation_type {
            AllocationType::Memory => MmapOptions::new().len(size).map_anon(),
            AllocationType::File => {
                let file = tempfile()?;
                file.set_len(size as u64)?;
                unsafe { MmapOptions::new().map_mut(&file) }
            }
        }?;

        let ptr = mmap.as_mut_ptr();
        info!("allocated to {:?} {}", ptr, self.total_size());
        self.mmaps.write().push(mmap);
        info!("done");
        Ok(ptr)
    }
}
