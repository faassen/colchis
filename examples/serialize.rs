use colchis::{Document, RoaringUsageBuilder};
use std::env;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    // Check for command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <json_file_path>", args[0]);
        return Ok(());
    }

    // Read from file
    let file_path = &args[1];
    let file = File::open(file_path)?;

    // Parse the JSON document
    match Document::parse::<RoaringUsageBuilder, _>(file) {
        Ok(document) => {
            // serializing it again to stdout
            document
                .serialize(&mut io::stdout())
                .expect("Failed to serialize document");
        }
        Err(err) => {
            println!("Error parsing JSON: {:?}", err);
        }
    }

    Ok(())
}
