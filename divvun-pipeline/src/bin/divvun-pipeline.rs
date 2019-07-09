use clap::{App, Arg, crate_version};

fn main() {
    let pipeline = "pipeline";

    let matches = App::new("divvun-pipeline")
        .version(crate_version!())
        .author("projektir <oprojektir@gmail.com>")
        .about("Asynchronous parallel pipeline for text processing.")
            .arg(Arg::with_name(pipeline)
                .help("The .zpipe file with the requested pipeline flow and required resources")
                .index(1)
            )
        .get_matches();

    if let Some(pipeline_file) = matches.value_of(pipeline) {
        println!("Offered file {}", pipeline_file);
    }
}
