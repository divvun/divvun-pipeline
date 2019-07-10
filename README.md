Pipeline so parallel, very async, wow.

Can run pipeline with:

`cargo run --bin divvun-pipeline -- myfile.zpipe`

`cargo run --bin divvun-pipeline divvun-pipeline/tests/pipeline.zpipe`

To generate a 0 compression pipeline zip file (on Unix):

`zip -0 pipeline.zpipe pipeline.json`

To run tests:

On Mac:

`./test.sh`

On Linux:

`./test-linux.sh`

On Windows:

Exercise for the reader, but probably modify one of the other files to refer to `.dll` files.
