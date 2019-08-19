use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use divvun_pipeline::{module::*, resources::ResourceRegistry};

pub fn setup_test_registry(
    allocation_type: AllocationType,
) -> (ModuleRegistry, Arc<ModuleAllocator>, Arc<ResourceRegistry>) {
    let _ = env_logger::builder().is_test(true).try_init();

    let allocator = Arc::new(ModuleAllocator::new(allocation_type));
    let resources = Arc::new(ResourceRegistry::new());

    let mut registry = ModuleRegistry::new(allocator.clone(), resources.clone()).unwrap();

    registry.add_search_path(&get_test_module_search_path());

    (registry, allocator, resources)
}

pub fn get_test_module_search_path() -> PathBuf {
    let mut d = get_project_root();
    d.push("target");
    d.push("modules");
    d
}

pub fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}
