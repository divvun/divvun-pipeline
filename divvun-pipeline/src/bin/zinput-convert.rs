use std::io;

use clap::{crate_version, App, Arg};

use divvun_schema::{capnp_message, string_capnp::string};

fn main() {
    let matches = App::new("zinput-convert")
        .version(crate_version!())
        .author("projektir <oprojektir@gmail.com>")
        .about("Utility for converting input into a divvun-pipeline compatible format.")
        .arg(
            Arg::with_name("text")
                .value_name("PATH")
                .help("Convert text")
                .short("t")
                .long("text")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    if let Some(text) = matches.value_of("text") {
        let msg = capnp_message!(string::Builder, builder => {
            builder.set_string(text);
        });

        capnp::serialize::write_message(&mut io::stdout(), &msg).unwrap();
    }
}
