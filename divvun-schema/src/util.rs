use crate::interface;
use capnp::message::TypedReader;
use std::{error::Error, io::Cursor, result::Result, slice, vec::Vec};

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

#[macro_export]
macro_rules! module_metadata {
    (@field $b:ident name $v:expr) => {
        $b.set_module_name($v);
    };

    (@field $b:ident version $v:expr) => {
        $b.set_module_version($v);
    };

    (@field $b:ident commands { $($ident:expr => [$($ip:ty),* $(,)*] => $op:ty),* $(,)* }) => {
        let mut commands = $b.init_commands(module_metadata!(@count $($ident),*));
        module_metadata!(@commands commands 0; $($ident => [$($ip),*] => $op),* ,);
    };

    // Count
    (@count $i:tt, $($e:tt),*) => (
        1 + module_metadata!(@count $($e),*)
    );

    (@count $i:tt) => (1);

    // Commands
    (@commands $c:ident $i:expr; $ident:expr => [$($ip:ty),*] => $op:ty, $($tail:tt)*) => (
        {
            use capnp::traits::HasTypeId;
            let mut command = $c.reborrow().get($i);
            command.set_name($ident);
            command.set_output(<$op>::type_id());
            let inputs = command.init_inputs(module_metadata!(@count $($ip),*));
            {
                module_metadata!(@inputs inputs 0; $($ip),* ,);
            }
        }
        module_metadata!(@commands $c ($i+1); $($tail)*);
    );

    (@commands $c:ident $i:expr; ) => ();

    // Inputs
    (@inputs $c:ident $i:expr; $ip:ty, $($tail:tt)*) => (
        {
            $c.reborrow().set($i, <$ip>::type_id());
        }
        module_metadata!(@inputs inputs ($i+1); $($tail)*);
    );

    (@inputs $c:ident $i:expr; ) => ();

    // Main
    (
        $(
            $f:ident
            :
            $v:tt
        ),* $(,)*
    ) => {{
        capnp_message!(divvun_schema::module_metadata_capnp::module_metadata::Builder, builder => {
            $(
                module_metadata!(@field builder $f $v);
            )*
        })
    }};
}

pub fn read_message<T: for<'a> capnp::traits::Owned<'a>>(
    input: *const u8,
    input_size: usize,
) -> Result<TypedReader<capnp::serialize::OwnedSegments, T>, Box<dyn Error>> {
    let slice = unsafe { std::slice::from_raw_parts(input, input_size) };
    let mut cursor = Cursor::new(slice);
    let message =
        capnp::serialize::read_message(&mut cursor, capnp::message::ReaderOptions::new())?;
    Ok(message.into())
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
