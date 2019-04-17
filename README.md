# Shosearch

Shosearch is a tool that queries Shodan database and puts results in mongodb.

It's best to use this tool together with shoview, which retrieves data from mongodb and generates reports.

# Basic usage

Make sure mongod is running. (To install on Mac: `brew install mongodb`, then `mongod` to start daemon).

Make sure `cargo` command is working. To install cargo, go to https://rustup.rs/ and follow the instructions.

You can run `cargo run -- -h` to see help.

Usage examples:
* `cargo run -- -a [api-key] -q nginx` will perform 'nginx' query against shodan API, using default mongodb collection, just 100 first entries
* `cargo run -- -a [api-key] -q nginx -c my_coll` will save results to a different collection
* `cargo run -- -a [api-key] -q apache2 -p 2` will use 'apache2' query and get 2 pages (200 entries)
* `cargo run -- -a [api-key] -q "apache2 country:US"` you can also use more complicated queries


# Building

Shosearch is written in Rust. Can be build using *cargo*, like any other Rust program.

To install cargo, go to https://rustup.rs/ and follow the instructions.

To build shosearch, run
```
cargo build --release
```
in repo's main directory.

The binary will be in target/release folder.
