use std::sync::Arc;

pub mod module;
pub mod pipeline;

use module::*;
use pipeline::*;

fn main() {
    env_logger::init();

    let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
    let registry = ModuleRegistry::new(allocator).unwrap();

    let mut module = registry.get_module("reverse_string").unwrap();
    let inputs: Vec<*const u8> = Vec::new();
    let input_sizes: Vec<usize> = Vec::new();

    println!("calling init");
    let result = module.call_init();
    let result = module.call_run("reverse", inputs, input_sizes);
    println!("result {:?}", result);
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_test() {
        let lol = Pipeline {
            root_item: PipelineItem::Serial(vec![
                PipelineItem::Single(PipelineCall {
                    module_name: "reverse_string".into(),
                    command_name: "reverse".into(),
                }),
                PipelineItem::Single(PipelineCall {
                    module_name: "reverse_string".into(),
                    command_name: "reverse".into(),
                }),
            ]),
        };
    }
}
