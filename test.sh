cargo build
mkdir -p target/modules
cp target/debug/libreverse_string.dylib target/modules/reverse_string.dylib
cp target/debug/libdo_things_strings.dylib target/modules/do_things_strings.dylib
cp target/debug/libconcat_strings.dylib target/modules/concat_strings.dylib
cp target/debug/libdivvun_pipeline_hfst.dylib target/modules/hfst.dylib
cp target/debug/libdivvun_pipeline_cg3.dylib target/modules/cg3.dylib
RUST_LOG=divvun_pipeline=info cargo test -- --test-threads 1 --nocapture
