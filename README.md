Pipeline so parallel, very async, wow.

Can run pipeline with:

`cargo run --bin divvun-pipeline -- myfile.zpipe`

`cargo run --bin divvun-pipeline divvun-pipeline/tests/pipeline.zpipe`

To test text input and output:

`cargo run --bin zinput-convert -- --text "this is my awesome string that should come back the same" | cargo run --bin zoutput-convert`

Input, output, divvun-pipeline:

`cargo run --bin zinput-convert -- --text "this is my awesome string that should come back the same" | cargo run --bin divvun-pipeline divvun-pipeline/tests/pipeline.zpipe`

To generate a 0 compression pipeline zip file (on Unix):

`zip -0 -r pipeline.zpipe pipeline.json yummy_resource`

If you just do `zip -0 -r pipeline.zpipe unzipped`, it will have the actual folder `unzipped` there, which is not supported

To run tests:

On Mac:

`./test.sh`

On Linux:

`./test-linux.sh`

On Windows:

Exercise for the reader, but probably modify one of the other files to refer to `.dll` files.

# Hfst Module
See modules/hfst/README.md about getting the latest hfst binaries to compile. Get the se.zcheck file from somewhere and extract to the folder se_zcheck.

Unignore the test in divvun-pipeline/tests/hfst.rs and run that after compiling & copying everything with ./test.sh
