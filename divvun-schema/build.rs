use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    let mut schema_file_path: PathBuf = PathBuf::from(env::var("OUT_DIR").unwrap());
    schema_file_path.push("capnp_schema.rs");

    let mut schema_file = File::create(schema_file_path).unwrap();

    let schema_dir = "schema";

    let mut command = capnpc::CompilerCommand::new();
    command.src_prefix(schema_dir);

    for entry in std::fs::read_dir(schema_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().unwrap() == "capnp" {
            let capnp_name = format!("{}_capnp", path.file_stem().unwrap().to_str().unwrap());

            command.file(path);

            writeln!(
                schema_file,
                r#"pub mod {file_name} {{
    include!(concat!(env!("OUT_DIR"), "/{file_name}.rs"));
}}"#,
                file_name = capnp_name
            ).unwrap();
        }
    }

    command.run().expect("compiling schema");
}
