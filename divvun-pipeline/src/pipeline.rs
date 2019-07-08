use std::sync::Arc;
use std::error::Error;

use futures::future::{join_all, FutureExt};
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

type PipelineType = Arc<Vec<Arc<String>>>;

impl Pipeline {
    pub async fn run(&self, registry: Arc<ModuleRegistry>, input: PipelineType) -> Result<PipelineType, PipelineError> {
        self.root.run(registry, input).await
    }
}

impl PipelineNodeSerial {
    async fn run(&self, registry: Arc<ModuleRegistry>, input: PipelineType) -> Result<PipelineType, PipelineError> {
        match self {
            PipelineNodeSerial::SerialSingle(command) => {
                process_single(registry, command, input)
            }
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
    async fn run(&self, registry: Arc<ModuleRegistry>, input: PipelineType) -> Result<PipelineType, PipelineError> {
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
                        let mut vector: Vec<Arc<String>> = Vec::new();

                        match result_vec {
                            Ok(arced_vec) => {
                                for string in arced_vec.iter() {
                                    vector.push(string.clone());
                                }
                            },
                            Err(e) => {
                                errors.push(e)
                            }
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

fn process_single(registry: Arc<ModuleRegistry>, command: &PipelineCommand, input: PipelineType) -> Result<PipelineType, PipelineError> {
    // TODO: fix this mess
    let module = registry.get_module(&command.module).unwrap();
    let metadata_mutex = module.metadata().as_ref().unwrap();
    let metadata_lock = metadata_mutex.lock().unwrap();
    let metadata = metadata_lock.get().unwrap();

    let module_name = metadata.get_module_name().unwrap();
    let commands = metadata.get_commands().unwrap();

    Ok(Arc::new(vec![Arc::new(format!(
        "|{} {} ran on input:{:?}\n|",
        &module_name, &command.command, input
    ))]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::module::{ModuleAllocator, AllocationType};

    #[test]
    fn init() {
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

    #[ignore]
    #[runtime::test]
    async fn pipeline_run() {
        let json_nodes = json!([
            { "module": "example-mod-1", "command": "tokenize"},
            [
                [
                    { "module": "example-mod-2", "command": "convertSomehow" },
                    { "module": "example-mod-1", "command": "grammarCheck" }
                ],
                { "module": "divvunspell", "command": "suggest" }
            ],
            { "module": "example-mod-4", "command": "convertToJson" }
        ]);

        let json_str = serde_json::to_string(&json_nodes).unwrap();
        let pipeline: Pipeline = Pipeline {
            root: serde_json::from_str(&json_str).unwrap(),
        };

        let allocator = Arc::new(ModuleAllocator::new(AllocationType::Memory));
        let registry = Arc::new(ModuleRegistry::new(allocator).unwrap());

        let result = pipeline
            .run(
                Arc::clone(&registry),
                Arc::new(vec![Arc::new("initial string".to_owned())]),
            )
            .await;

        println!("{:?}", result);
    }
}
