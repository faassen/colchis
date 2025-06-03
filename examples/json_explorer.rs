use colchis::{BitpackingUsageBuilder, Document};
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
    println!("Reading JSON from file: {}", file_path);
    let file = File::open(file_path)?;
    // do not use a buffer, get a reader to avoid unnecessary memory usage
    // no need for a bufreader as struson handles buffering internally
    match Document::parse::<BitpackingUsageBuilder, _>(&file) {
        Ok(document) => {
            // Display document information
            let heap_size = document.heap_size();
            println!("\n===== JSON Document Analysis =====");
            println!(
                "Heap size: {} bytes ({:.4} mb)",
                heap_size,
                heap_size as f64 / (1024.0 * 1024.0)
            );
        }
        Err(err) => {
            println!("Error parsing JSON: {:?}", err);
        }
    }

    Ok(())
}
