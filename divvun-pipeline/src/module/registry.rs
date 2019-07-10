use log::info;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;

use super::{Module, ModuleAllocator};
use crate::resources::ResourceRegistry;

#[derive(Debug)]
pub enum ModuleLoadError {
    LoadFailed(Vec<Box<dyn Error>>),
}

impl fmt::Display for ModuleLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModuleLoadError::LoadFailed(ref parent_errors) => {
                writeln!(f, "Module load failed:");
                for error in parent_errors {
                    writeln!(f, "{}", error);
                }
            }
        };

        Ok(())
    }
}

impl Error for ModuleLoadError {}

/// A module registry that is responsible of finding pipeline modules to load, loading them
/// and properly initializing them.
pub struct ModuleRegistry {
    allocator: Arc<ModuleAllocator>,
    resource_registry: Arc<ResourceRegistry>,
    search_paths: HashSet<PathBuf>,
    registry: RwLock<HashMap<String, Arc<Module>>>,
}

impl ModuleRegistry {
    /// Create a new module registry with the default search path of '{current_dir}/modules'
    /// and using the passed in allocator to initialize all modules loaded in the future
    pub fn new(
        allocator: Arc<ModuleAllocator>,
        resource_registry: Arc<ResourceRegistry>,
    ) -> Result<ModuleRegistry, Box<dyn Error>> {
        let mut search_paths = HashSet::new();

        // Add default search path
        let mut path = PathBuf::from(std::env::current_dir()?);
        path.push("modules");
        search_paths.insert(path);

        Ok(ModuleRegistry {
            allocator,
            resource_registry,
            search_paths,
            registry: RwLock::new(HashMap::new()),
        })
    }

    /// Add a search path to be searched when loading modules
    pub fn add_search_path(&mut self, path: &Path) {
        self.search_paths.insert(path.into());
    }

    /// Enumerate the registry's search paths and try to load the module with the given name.
    /// If found, initializes the module and returns it. Alternatively returns a list of
    /// errors for each attempted load (if there are multiple search paths).
    pub fn get_module(&self, module_name: &str) -> Result<Arc<Module>, Box<dyn Error>> {
        info!("module name: {}", module_name);

        {
            let lock = self.registry.read();

            if lock.contains_key(module_name) {
                return Ok(Arc::clone(lock.get(module_name).unwrap()));
            }
        }

        let ext = if cfg!(target_os = "macos") {
            "dylib"
        } else if cfg!(target_os = "windows") {
            "dll"
        } else {
            "so"
        };

        // TODO: actually store in registry
        let load_paths = self
            .search_paths
            .iter()
            .map(|path| path.join(format!("{}.{}", module_name, ext)));

        let mut errors = Vec::new();
        for path in load_paths {
            info!("trying to load from {}", path.display());
            let module = Module::load(
                self.allocator.clone(),
                self.resource_registry.clone(),
                &path,
            );
            match module {
                Ok(module) => {
                    let mut lock = self.registry.write();
                    lock.insert(module_name.to_owned(), Arc::clone(&module));

                    return Ok(module);
                }
                Err(err) => errors.push(err),
            };
        }

        Err(ModuleLoadError::LoadFailed(errors).into())
    }
}
