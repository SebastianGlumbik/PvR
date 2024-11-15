fn main() {
    // TODO #1: implement a simple version of grep
    // Your program should go through a specified directory recursively, read the contents of all
    // files and print all lines (+ their locations) that contain a specified substring.
    // You don't have to use regexes, a normal substring search will work just fine.
    // You can use a crate to iterate directories (e.g. `walkdir`) if you want, or just code the
    // traversal by hand.
    // You can download e.g. the cargo repository (`git clone https://github.com/rust-lang/cargo)
    // to have some data to search through, and grep e.g. for Rust keywords in it.

    // TODO #2: add a command-line interface
    // Use the `clap` crate to add a simple CLI to your program, which will be used to select which
    // directory (or file) should be searched, and what substring should be searched.

    // TODO #3: add JSON output
    // Use the `serde` and `serde_json` crates to print the output in JSON, so that it can be
    // handled programmatically.
    // Use the CLI to select if the program should print the output in human-readable form or in
    // JSON.

    // TODO #4: parallelize the search
    // Perform search across files in parallel.
    // Perform search across lines/parts of files in parallel.
}
