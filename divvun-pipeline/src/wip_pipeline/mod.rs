mod module;
mod pipeline;

use std::sync::Arc;

use futures::future::FutureExt;
use serde_json::json;

use pipeline::Pipeline;

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

    let registry = pipeline.load_modules();

    let result = pipeline
        .run(
            Arc::clone(&registry),
            Arc::new(vec![Arc::new("initial string".to_owned())]),
        )
        .await;

    println!("{:?}", result);
}
