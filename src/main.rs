use std::fs;

fn main() {
    match fs::read_to_string("My Clippings.txt") {
        Ok(contents) => {
            println!("Clippings: \n{}", contents);
        }
        Err(error) => {
            eprintln!("Error to read file{}", error);
        }
    }
}
