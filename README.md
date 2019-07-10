Can run pipeline with:

`cargo run --bin divvun-pipeline -- myfile.zpipe`

`cargo run --bin divvun-pipeline divvun-pipeline/tests/pipeline.zpipe`

To generate a 0 compression pipeline zip file (on Unix):

`zip -0 pipeline.zpipe pipeline.json`
