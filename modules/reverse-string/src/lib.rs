use capnp::message::ReaderOptions;
use capnp::serialize;
use divvun_schema::capnp_message;
use divvun_schema::interface::{self, PipelineInterface};
use divvun_schema::string_capnp::string;
use divvun_schema::util;
use lazy_static::lazy_static;
use std::ffi::CStr;
use std::io::Cursor;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const PipelineInterface) -> bool {
    println!("pipeline_init reverse-string");
    interface::initialize(interface)
}

#[no_mangle]
pub extern "C" fn pipeline_run(
    command: *const c_char,
    input_count: usize,
    input: *const *const u8,
    input_sizes: *const usize,
    output: *mut *const u8,
    output_size: *mut usize,
) -> bool {
    println!("Hello, world from module!");
    let command = unsafe { CStr::from_ptr(command) }.to_string_lossy();
    println!("command = {}, input_count = {}", command, input_count);

    let input_sizes = unsafe { std::slice::from_raw_parts(input_sizes, input_count) };
    let input = unsafe { std::slice::from_raw_parts(input, input_count) };

    match &*command {
        "reverse" => {
            for i in 0..input_count {
                let slice = unsafe { std::slice::from_raw_parts(input[i], input_sizes[i]) };
                let mut cursor = Cursor::new(slice);
                let message = serialize::read_message(&mut cursor, ReaderOptions::new()).unwrap();
                let string = message.get_root::<string::Reader>().unwrap();
                let result: String = string.get_string().unwrap().chars().rev().collect();
                println!(
                    "receives input {}, returning {}",
                    string.get_string().unwrap(),
                    result
                );

                util::output_message(
                    output,
                    output_size,
                    capnp_message!(string::Builder, builder => {
                        builder.set_string(&result);
                    }),
                )
                .unwrap();

                println!("returning from reverse");

                return true;
            }

            util::output_message(
                output,
                output_size,
                divvun_schema::capnp_error!(
                    divvun_schema::error_capnp::pipeline_error::ErrorKind::ModuleError,
                    "no input provided"
                ),
            )
            .unwrap();

            false
        }
        _ => {
            util::output_message(
                output,
                output_size,
                divvun_schema::capnp_error!(
                    divvun_schema::error_capnp::pipeline_error::ErrorKind::UnknownCommand,
                    &format!("unknown command {}", command)
                ),
            )
            .unwrap();

            false
        }
    }
}

// // Pseudocode macro for declaring module metadata

// static metadata: ModuleMetadata = module_metadata! {
//   name: "reverse-string",
//   commands: {
//     "reverse" => { inputs: [ReverseInput], output: TokenizedString },
//     "lol" => { inputs: [ReverseInput], output: TokenizedString }
//   }
// }

// struct ReverseInput {
//     string: InputString,
//     audio: InputAudio
// }

// macro_rules! module_metadata {
//     (@field $b:ident name $v:expr) => {
//         $b.set_module_name($v);
//     };

//     (@field $b:ident commands { $($ident:expr => $block:block),* $(,)* }) => {
//         let mut commands = $b.init_commands(module_metadata!(@commands_count $($ident),*));
//         // stringify!($($t)*);
//         module_metadata!(@commands commands 0, $($ident => $block),*);
//     };

//     // Commands
//     (@commands_count $i:expr, $($e:expr),*) => (
//         1 + module_metadata!(@commands_count $($e),*)
//     );

//     (@commands_count $i:expr) => (1);

//     (@commands $c:ident $i:expr, $ident:expr => $block:block, $($t:tt)*) => (
//         {
//             let mut command = $c.reborrow().get($i);
//             let inputs = command.init_accepted_inputs(1);
//             {
//                 // use capnp::traits::HasTypeId;
//                 // inputs.reborrow().set(0, divvun_schema::string_capnp::string::Builder::type_id());
//             }
//         }
//         stringify!($($t)*);
//         module_metadata!(@commands $c ($i+1), $($t)*);
//     );

//     (@commands $c:ident $i:expr,) => ();

//     (
//         $(
//             $f:ident
//             :
//             $v:tt
//         ),* $(,)*
//     ) => {{
//         capnp_message!(divvun_schema::module_metadata_capnp::module_metadata::Builder, builder => {
//             $(
//                 module_metadata!(@field builder $f $v);
//                 // stringify!($f);
//                 stringify!($v);
//             )*
//         });
//     }};

//     // ($b: tt) => {

//     // };
// }

#[no_mangle]
pub extern "C" fn pipeline_info(metadata: *mut *const u8, metadata_size: *mut usize) -> bool {
    // module_metadata! {
    //     name: "reverse-string",
    //     commands: {
    //         "reverse" => {},
    //         "lol" => {},
    //     }
    // };

    lazy_static! {
        static ref MESSAGE: Vec<u8> = divvun_schema::util::message_to_vec(
            capnp_message!(divvun_schema::module_metadata_capnp::module_metadata::Builder, builder => {
                builder.set_module_name("reverse-string");
                let mut commands = builder.init_commands(1);
                {
                    use capnp::traits::HasTypeId;
                    let mut command = commands.reborrow().get(0);
                    command.set_name("reverse");
                    command.set_output(divvun_schema::string_capnp::string::Builder::type_id());
                    let inputs = command.init_inputs(1);
                    {
                        inputs.reborrow().set(0, divvun_schema::string_capnp::string::Builder::type_id());
                    }
                }
            }),
        ).unwrap();
    }

    unsafe {
        *metadata = MESSAGE.as_ptr();
        *metadata_size = MESSAGE.len();
    }

    true
}
