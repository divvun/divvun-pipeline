use crate::interface;
use std::io::Cursor;
use std::result::Result;
use std::slice;
use std::vec::Vec;

/// Create a message with a capnp structure of the passed in builder type and
/// invoke the closure.
#[macro_export]
macro_rules! capnp_message {
    ($i:ty, $v:ident => $b:block) => {{
        let mut message = capnp::message::Builder::new_default();
        {
            let mut $v = message.init_root::<$i>();
            $b
        }

        message
    }};
}

/// Create an error message with the passed in kind and message text
#[macro_export]
macro_rules! capnp_error {
    ($k:expr, $m:expr) => {{
        divvun_schema::capnp_message!(divvun_schema::error_capnp::pipeline_error::Builder, builder => {
            builder.set_kind($k);
            builder.set_message($m);
        })
    }};
}

/// This uses the interface allocator to allocate enough memory for the passed in message
/// and then writes it to the passed in output. Best used with capnp_message or capnp_error
/// to produce an output.
pub fn output_message<A: capnp::message::Allocator>(
    output: *mut *const u8,
    output_size: *mut usize,
    message: capnp::message::Builder<A>,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized_size = capnp::serialize::compute_serialized_size_in_words(&message)
        * std::mem::size_of::<capnp::Word>();
    let memory = interface::allocate(serialized_size).expect("memory to be allocated");
    let slice = unsafe { slice::from_raw_parts_mut(memory, serialized_size) };
    let mut cursor = Cursor::new(slice);
    capnp::serialize::write_message(&mut cursor, &message)?;
    unsafe {
        *output = memory;
        *output_size = serialized_size;
    }

    Ok(())
}

/// Serializes a message into a Vec<u8>, should only be used for tests or building an input
pub fn message_to_vec<A: capnp::message::Allocator>(
    message: capnp::message::Builder<A>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut vec = Vec::new();
    capnp::serialize::write_message(&mut vec, &message)?;
    Ok(vec)
}
