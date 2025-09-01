use std::env;
use std::process;

use kindlr::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        eprintln!("\nUsage: kindlr <file_path>");
        process::exit(1);
    });

    if let Err(e) = kindlr::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
