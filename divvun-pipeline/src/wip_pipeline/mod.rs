mod pipeline;

use std::sync::Arc;

use futures::future::FutureExt;
use serde_json::json;

use pipeline::Pipeline;
use super::module::{Module, ModuleRegistry};

#[runtime::test]
async fn pipeline_test() {
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
