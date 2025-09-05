use std::error::Error;
use std::fmt;
use std::fs;
use std::io;

pub mod parser;

#[derive(Debug)]
pub enum KindlrError {
    Io(io::Error),
    Parse(parser::ParseError),
    Config(String),
}

impl fmt::Display for KindlrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KindlrError::Io(err) => write!(f, "IO error: {}", err),
            KindlrError::Parse(msg) => write!(f, "Parse error: {}", msg),
            KindlrError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for KindlrError {}

impl From<io::Error> for KindlrError {
    fn from(err: io::Error) -> Self {
        KindlrError::Io(err)
    }
}

impl From<parser::ParseError> for KindlrError {
    fn from(err: parser::ParseError) -> Self {
        KindlrError::Parse(err)
    }
}

/// Application configuration
pub struct Config {
    pub file_path: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, KindlrError> {
        args.next();

        let file_path = args
            .next()
            .ok_or_else(|| KindlrError::Config("Missing file path argument".to_string()))?;

        // let command = args
        //     .next()
        //     .ok_or_else(|| KindlrError::Config("Didn't get a command string".to_string()))?;

        Ok(Config { file_path })
    }
}

pub fn run(config: Config) -> Result<(), KindlrError> {
    let contents = fs::read_to_string(config.file_path)?;

    let clippings = parser::parse_clippings(&contents)?;

    for (i, clipping) in clippings.iter().enumerate() {
        println!("Clipping #{}:", i + 1);
        println!("{}", clipping);
        println!();
    }

    println!("Total clippings: {}", clippings.len());

    Ok(())
}
