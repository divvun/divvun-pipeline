use std::io::{self, Write};

use divvun_schema::string_capnp::string;

fn main() {
    let message =
        capnp::serialize::read_message(&mut io::stdin(), capnp::message::ReaderOptions::new())
            .unwrap();

    let string = message
        .get_root::<string::Reader>()
        .unwrap()
        .get_string()
        .unwrap();

    io::stdout().write_all(string.as_bytes()).unwrap();
}
