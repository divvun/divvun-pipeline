use std::{path::Path, sync::Arc};

use divvun_pipeline::{
    module::*,
    pipeline::*,
    resources::{LoadableResource, Resource, ResourceRegistry},
};
use divvun_schema::{string_capnp::string, util};

mod common;

#[test]
// #[ignore]
fn hfst_module() {
    let (registry, allocator, resources) = common::setup_test_registry(AllocationType::Memory);
    let mut res_path = common::get_project_root();
    res_path.push("se_zcheck/tokeniser-gramcheck-gt-desc.pmhfst");
    resources.add_resource(
        "pmatch_file",
        LoadableResource::from(Resource::new_file(&res_path)),
    );

    let module = registry.get_module("hfst").unwrap();

    let text = util::message_to_vec(divvun_schema::capnp_message!(string::Builder, builder => {
        builder.set_string("Hello world what is going on, please correct me");
    }))
    .unwrap();

    let inputs: Vec<*const u8> = vec![text.as_ptr()];
    let input_sizes: Vec<usize> = vec![text.len()];

    let parameters = vec!["pmatch_file".to_string()];
    let result = module.call_run("tokenize", Some(&parameters), inputs, input_sizes);
    assert!(result.is_ok());

    let result = result.unwrap();

    let slice = unsafe { std::slice::from_raw_parts(result.output, result.output_size) };
    let mut cursor = std::io::Cursor::new(slice);

    let message =
        capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new()).unwrap();
    let text = message
        .get_root::<divvun_schema::string_capnp::string::Reader>()
        .unwrap();

    assert_eq!(text.get_string().unwrap(), "\"<Hello>\"\n\t\"heallat\" Ex/V Ex/IV Der/PassS V IV Ind Prs ConNeg <W:0.0>\n\t\"heallat\" Ex/V Ex/IV Der/PassS V IV Ind Prs Sg3 <W:0.0>\n\t\"heallat\" V IV Imprt ConNegII <W:0.0>\n: \n\"<world>\"\n\t\"world\" ?\n: \n\"<what>\"\n\t\"what\" ?\n: \n\"<is>\"\n\t\"is\" ?\n: \n\"<going>\"\n\t\"going\" ?\n: \n\"<on>\"\n\t\"on\" Adv <W:0.0>\n\"<,>\"\n\t\",\" CLB <W:0.0>\n: \n\"<please>\"\n\t\"please\" ?\n: \n\"<correct>\"\n\t\"correct\" ?\n: \n\"<me>\"\n\t\"me\" ?\n");
}
