cargo build
cp target/debug/libreverse_string.so modules/reverse_string.so
cp target/debug/libdo_things_strings.so modules/do_things_strings.so
cp target/debug/libconcat_strings.so modules/concat_strings.so
RUST_LOG=divvun_pipeline=info cargo test -- --test-threads 1 --nocapture
