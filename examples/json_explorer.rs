use colchis::Document;
use std::env;
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    // Check for command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <json_file_path>", args[0]);
        return Ok(());
    }

    // Read from file
    let file_path = &args[1];
    println!("Reading JSON from file: {}", file_path);
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    // Parse the JSON document
    match Document::parse(&contents) {
        Ok(document) => {
            // Display document information
            println!("\n===== JSON Document Analysis =====");
            println!("Heap size: {} bytes", document.heap_size());
        }
        Err(err) => {
            println!("Error parsing JSON: {:?}", err);
        }
    }

    Ok(())
}
