use log::info;
use memmap::{Mmap, MmapOptions};
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};

use std::sync::Arc;

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use parking_lot::RwLock;
use std::collections::HashMap;

pub enum Resource {
    File { path: PathBuf, mmap: Option<Mmap> },
    Bytes(Vec<u8>),
}

impl Resource {
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        info!("load");
        match self {
            Resource::File { path, ref mut mmap } => {
                let file = File::open(path)?;
                *mmap = Some(unsafe { MmapOptions::new().map(&file)? });
                Ok(())
            }
            Resource::Bytes(_) => Ok(()),
        }
    }

    pub fn unload(&mut self) -> Result<(), Box<dyn Error>> {
        info!("unload");
        match self {
            Resource::File { ref mut mmap, .. } => {
                let _ = mmap.take();
                Ok(())
            }
            Resource::Bytes(_) => Ok(()),
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Resource::File { ref mmap, .. } => mmap.as_ref().map(|mmap| mmap.len()),
            Resource::Bytes(vec) => Some(vec.len()),
        }
    }

    pub fn as_ptr(&self) -> Option<*const u8> {
        match self {
            Resource::File { ref mmap, .. } => mmap.as_ref().map(|mmap| mmap.as_ptr()),
            Resource::Bytes(vec) => Some(vec.as_ptr()),
        }
    }
}

pub struct LoadableResource {
    resource: RwLock<Resource>,
    ref_counter: AtomicUsize,
}

impl LoadableResource {
    pub fn claim(&self) {
        let last_ref = self.ref_counter.fetch_add(1, Ordering::SeqCst);
        info!("claim {}", last_ref);
        if last_ref == 0 {
            self.resource
                .write()
                .load()
                .expect("load of resource failed");
        }
    }

    pub fn release(&self) {
        let last_ref = self.ref_counter.fetch_sub(1, Ordering::SeqCst);
        info!("release {}", last_ref);
        if last_ref == 1 {
            self.resource
                .write()
                .unload()
                .expect("unload of resource failed");
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.ref_counter.load(Ordering::SeqCst) > 0
    }
}

impl From<Resource> for LoadableResource {
    fn from(resource: Resource) -> Self {
        LoadableResource {
            resource: RwLock::new(resource),
            ref_counter: AtomicUsize::new(0),
        }
    }
}

pub struct ResourceHandle {
    loadable_resource: Arc<LoadableResource>,
}

impl ResourceHandle {
    pub fn size(&self) -> Option<usize> {
        self.loadable_resource.resource.read().size()
    }

    pub fn as_ptr(&self) -> Option<*const u8> {
        self.loadable_resource.resource.read().as_ptr()
    }
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        self.loadable_resource.release();
    }
}

pub struct ResourceRegistry {
    available: RwLock<HashMap<String, Arc<LoadableResource>>>,
}

impl ResourceRegistry {
    pub fn new() -> ResourceRegistry {
        ResourceRegistry {
            available: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_resource(&self, name: &str, resource: LoadableResource) {
        self.available
            .write()
            .insert(name.to_string(), Arc::new(resource));
    }

    pub fn get(&self, name: &str) -> Option<ResourceHandle> {
        let lock = self.available.read();
        let resource = lock.get(name)?;
        resource.claim();
        Some(ResourceHandle {
            loadable_resource: resource.clone(),
        })
    }

    pub fn loaded_resources_count(&self) -> usize {
        self.available
            .read()
            .values()
            .filter(|res| res.is_loaded())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resources_test() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut registry = ResourceRegistry::new();
        let my_data = "Hello".as_bytes();
        registry.add_resource(
            "lol",
            LoadableResource::from(Resource::Bytes(my_data.to_owned())),
        );

        assert_eq!(registry.loaded_resources_count(), 0);
        {
            let resource = registry.get("lol").unwrap();
            assert_eq!(resource.size().unwrap(), 5);
            assert_eq!(registry.loaded_resources_count(), 1);
        }
        assert_eq!(registry.loaded_resources_count(), 0);
    }
}
