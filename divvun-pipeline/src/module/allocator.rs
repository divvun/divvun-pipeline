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
        self.mmaps.read().iter().map(|m| m.len()).sum()
    }

    pub fn alloc(&self, size: usize) -> Result<*mut u8, Box<dyn Error>> {
        let mut mmap = match self.allocation_type {
            AllocationType::Memory => MmapOptions::new().len(size).map_anon(),
            AllocationType::File => {
                let file = tempfile()?;
                file.set_len(size as u64)?;
                unsafe { MmapOptions::new().map_mut(&file) }
            }
        }?;

        let ptr = mmap.as_mut_ptr();
        self.mmaps.write().push(mmap);
        Ok(ptr)
    }
}
