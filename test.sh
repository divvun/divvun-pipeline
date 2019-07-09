cargo build
cp target/debug/libreverse_string.dylib modules/reverse_string.dylib
cp target/debug/libdo_things_strings.dylib modules/do_things_strings.dylib
cp target/debug/libconcat_strings.dylib modules/concat_strings.dylib
RUST_LOG=divvun_pipeline=info cargo test -- --test-threads 1 --nocapture