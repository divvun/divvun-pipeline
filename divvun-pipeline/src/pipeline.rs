use std::sync::Arc;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::module::ModuleRegistry;

#[derive(Debug)]
pub enum PipelineError {
    NodeFailed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pipeline {
    pub root: PipelineNodeSerial,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineCommand {
    pub module: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PipelineNodeSerial {
    SerialSingle(PipelineCommand),
    SerialMultiple(Vec<PipelineNodeParallel>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PipelineNodeParallel {
    ParallelSingle(PipelineCommand),
    ParallelMultiple(Vec<PipelineNodeSerial>),
}

#[derive(Debug)]
pub struct PipelineData {
    pub data: *const u8,
    pub size: usize,
}

type PipelineType = Arc<Vec<Arc<PipelineData>>>;

unsafe impl Send for PipelineData {}
unsafe impl Sync for PipelineData {}

impl Pipeline {
    pub async fn run(
        &self,
        registry: Arc<ModuleRegistry>,
        input: PipelineType,
    ) -> Result<PipelineType, PipelineError> {
        // TODO: Validate here
        self.root.run(registry, input).await
    }
}

impl PipelineNodeSerial {
    async fn run(
        &self,
        registry: Arc<ModuleRegistry>,
        input: PipelineType,
    ) -> Result<PipelineType, PipelineError> {
        match self {
            PipelineNodeSerial::SerialSingle(command) => process_single(registry, command, input),
            PipelineNodeSerial::SerialMultiple(nodes) => {
                let mut input = input.clone();

                for node in nodes {
                    input = node.run(Arc::clone(&registry), input).await?;
                }

                Ok(input)
            }
        }
    }
}

impl PipelineNodeParallel {
    async fn run(
        &self,
        registry: Arc<ModuleRegistry>,
        input: PipelineType,
    ) -> Result<PipelineType, PipelineError> {
        match self {
            PipelineNodeParallel::ParallelSingle(command) => {
                process_single(registry, command, input)
            }
            PipelineNodeParallel::ParallelMultiple(nodes) => {
                let new_input = input.clone();

                let mut vector = Vec::new();
                for node in nodes {
                    vector.push(node.run(Arc::clone(&registry), new_input.clone()));
                }

                let future_results = join_all(vector).await;

                let mut errors: Vec<PipelineError> = Vec::new();

                let outputs = future_results
                    .into_iter()
                    .map(|result_vec| {
                        let mut vector: Vec<Arc<PipelineData>> = Vec::new();

                        match result_vec {
                            Ok(arced_vec) => {
                                for data in arced_vec.iter() {
                                    vector.push(data.clone());
                                }
                            }
                            Err(e) => errors.push(e),
                        }

                        vector
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                if errors.len() > 0 {
                    // TODO: Flatten the errors into something useful instead of just returning this
                    Err(PipelineError::NodeFailed)
                } else {
                    Ok(Arc::new(outputs))
                }
            }
        }
    }
}

fn process_single(
    registry: Arc<ModuleRegistry>,
    command: &PipelineCommand,
    input: PipelineType,
) -> Result<PipelineType, PipelineError> {
    // TODO: fix errors
    let module = registry.get_module(&command.module).unwrap();

    let mut ptr_vec = Vec::new();
    let mut size_vec = Vec::new();

    input.iter().for_each(|data| {
        ptr_vec.push(data.data);
        size_vec.push(data.size);
    });

    let output = module
        .call_run(&command.command, ptr_vec, size_vec)
        .unwrap();

    Ok(Arc::new(vec![Arc::new(PipelineData {
        data: output.output,
        size: output.output_size,
    })]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::{AllocationType, ModuleAllocator};
    use crate::resources::ResourceRegistry;
use std::path::Path;
use std::io::Cursor;
use divvun_schema::capnp_message;
use capnp::serialize;
use capnp::message::ReaderOptions;
use divvun_schema::string_capnp::string;
    use serde_json::json;

    #[test]
    fn init() {
        env_logger::init();

        let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
        let resources = Arc::new(ResourceRegistry::new());
        let mut registry = ModuleRegistry::new(allocator, resources).unwrap();
        registry.add_search_path(Path::new("../modules"));

        let module = registry.get_module("reverse_string").unwrap();
        let inputs: Vec<*const u8> = Vec::new();
        let input_sizes: Vec<usize> = Vec::new();

        println!("calling init");
        let result = module.call_run("reverse", inputs, input_sizes);
        println!("result {:?}", result);
        println!("Hello, world!");
    }

    #[runtime::test]
    async fn pipeline_run() {
        let _ = env_logger::builder().is_test(true).try_init();

        let json_nodes = json!([
            { "module": "reverse_string", "command": "reverse"},
              [
                 [
                    { "module": "do_things_strings", "command": "badazzle" },
                    { "module": "reverse_string", "command": "reverse" }
                 ],

                 { "module": "reverse_string", "command": "reverse" }
             ],
            { "module": "concat_strings", "command": "concat" }
        ]);

        let json_str = serde_json::to_string(&json_nodes).unwrap();
        let pipeline: Pipeline = Pipeline {
            root: serde_json::from_str(&json_str).unwrap(),
        };

        let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
        let resources = Arc::new(ResourceRegistry::new());
        let mut registry = ModuleRegistry::new(allocator, resources).unwrap();
        registry.add_search_path(Path::new("../modules"));
        let registry = Arc::new(registry);

        let msg = capnp_message!(string::Builder, builder => {
            builder.set_string("Hello world!");
        });

        let msg_vec = divvun_schema::util::message_to_vec(msg).unwrap();

        let result = pipeline
            .run(
                registry.clone(),
                Arc::new(vec![Arc::new(PipelineData {
                    data: msg_vec.as_ptr(),
                    size: msg_vec.len(),
                })]),
            )
            .await;

        let inter_output = result.unwrap();
        let output = inter_output.get(0).unwrap();

        let output_data = output.data;
        let output_size = output.size;

        println!("output: {:?}", output);
        let slice = unsafe { std::slice::from_raw_parts(output_data, output_size) };

        println!("slice: {:#?}", slice);
        let mut cursor = Cursor::new(slice);
        let message = serialize::read_message(&mut cursor, ReaderOptions::new()).unwrap();
        let string = message.get_root::<string::Reader>().unwrap();
        let result = string.get_string().unwrap();

        println!("{:?}", result);
    }
}
