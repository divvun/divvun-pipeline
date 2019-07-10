use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
