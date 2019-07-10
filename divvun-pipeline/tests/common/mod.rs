use std::path::Path;
use std::sync::Arc;

use divvun_pipeline::module::*;
use divvun_pipeline::pipeline::*;
use divvun_pipeline::resources::{LoadableResource, Resource, ResourceRegistry};

pub fn setup_test_registry(
    allocation_type: AllocationType,
) -> (ModuleRegistry, Arc<ModuleAllocator>, Arc<ResourceRegistry>) {
    let _ = env_logger::builder().is_test(true).try_init();

    let allocator = Arc::new(ModuleAllocator::new(allocation_type));
    let resources = Arc::new(ResourceRegistry::new());

    let mut registry = ModuleRegistry::new(allocator.clone(), resources.clone()).unwrap();
    registry.add_search_path(Path::new("../modules"));

    (registry, allocator, resources)
}
