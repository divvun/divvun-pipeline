use std::collections::HashSet;
use std::sync::Arc;

use futures::future::{join_all, FutureExt};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::module::{Module, ModuleRegistry};

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
    pub async fn run(&self, registry: Arc<ModuleRegistry>, input: PipelineType) -> PipelineType {
        self.root.run(registry, input).await
    }

    pub fn load_modules(&self) -> Arc<ModuleRegistry> {
        let mut module_registry = ModuleRegistry {
            registry: HashMap::new(),
        };

        let modules = self.root.module_names();

        for module in modules {
            module_registry.registry.insert(
                module.to_owned(),
                Module {
                    name: module.to_owned(),
                    output: format!("{} output", module),
                },
            );
        }

        Arc::new(module_registry)
    }
}

impl PipelineNodeSerial {
    async fn run(&self, registry: Arc<ModuleRegistry>, input: PipelineType) -> PipelineType {
        match self {
            PipelineNodeSerial::SerialSingle(command) => {
                let module = registry.registry.get(&command.module).unwrap();

                Arc::new(vec![Arc::new(format!(
                    "|{} {} ran on input:{:?}\n|",
                    module.name, command.command, input
                ))])
            }
            PipelineNodeSerial::SerialMultiple(nodes) => {
                let mut input = input.clone();

                for node in nodes {
                    input = node.run(Arc::clone(&registry), input).await;
                }

                input
            }
        }
    }

    pub fn module_names(&self) -> HashSet<String> {
        match self {
            PipelineNodeSerial::SerialSingle(command) => {
                let mut names = HashSet::new();
                names.insert(command.module.to_string());
                names
            }
            PipelineNodeSerial::SerialMultiple(nodes) => nodes
                .iter()
                .map(|node| node.module_names())
                .flatten()
                .collect(),
        }
    }
}

impl PipelineNodeParallel {
    async fn run(&self, registry: Arc<ModuleRegistry>, input: PipelineType) -> PipelineType {
        match self {
            PipelineNodeParallel::ParallelSingle(command) => {
                let module = registry.registry.get(&command.module).unwrap();

                Arc::new(vec![Arc::new(format!(
                    "|{} {} ran on input:{:?}\n|",
                    module.name, command.command, input
                ))])
            }
            PipelineNodeParallel::ParallelMultiple(nodes) => {
                let new_input = input.clone();

                let mut vector = Vec::new();
                for node in nodes {
                    vector.push(node.run(Arc::clone(&registry), new_input.clone()));
                }

                let future_results = join_all(vector).await;

                let results = future_results
                    .into_iter()
                    .map(|arced_vec| {
                        let arced_vec_iter = arced_vec.iter();
                        let mut vector: Vec<Arc<String>> = Vec::new();

                        for string in arced_vec_iter {
                            vector.push(string.clone());
                        }

                        vector
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                Arc::new(results)
            }
        }
    }

    pub fn module_names(&self) -> HashSet<String> {
        match self {
            PipelineNodeParallel::ParallelSingle(command) => {
                let mut names = HashSet::new();
                names.insert(command.module.to_string());
                names
            }
            PipelineNodeParallel::ParallelMultiple(nodes) => nodes
                .iter()
                .map(|node| node.module_names())
                .flatten()
                .collect(),
        }
    }
}
