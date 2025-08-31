use std::error::Error;
use std::fs;

pub struct Config {
    pub file_path: String,
    pub command: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path string"),
        };

        let command = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a command string"),
        };

        Ok(Config { file_path, command })
    }
}

pub fn read_clippings(config: Config) -> Result<(), Box<dyn Error>> {
    match fs::read_to_string(config.file_path) {
        Ok(contents) => {
            println!("Clippings: \n{}", contents);
        }
        Err(error) => {
            eprintln!("Error to read file{}", error);
        }
    }

    Ok(())
}
