use colchis::Document;
use std::env;
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    // Check for command line arguments
    let args: Vec<String> = env::args().collect();

    let json_content = if args.len() > 1 {
        // Read from file
        let file_path = &args[1];
        println!("Reading JSON from file: {}", file_path);
        let mut file = File::open(file_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        contents
    } else {
        // Use sample JSON if no file is provided
        println!("Using sample JSON document (no file provided)");
        let sample_json = r#"{
            "name": "JSON Explorer Example",
            "description": "A more detailed example of JSON parsing with colchis",
            "version": 2.0,
            "active": true,
            "features": ["parsing", "heap analysis", "memory efficiency"],
            "stats": {
                "creation_date": "2023-10-25",
                "last_updated": "2024-05-01",
                "measurements": [1.2, 3.4, 5.6, 7.8],
                "config": {
                    "debug": true,
                    "optimize": false,
                    "max_depth": 10
                }
            },
            "null_value": null
        }"#
        .as_bytes()
        .to_vec();
        sample_json
    };

    // Parse the JSON document
    match Document::parse(&json_content) {
        Ok(document) => {
            // Display document information
            println!("\n===== JSON Document Analysis =====");
            println!("Heap size: {} bytes", document.heap_size());

            // You could implement more document exploration here
            // For example, traverse the document structure, extract values, etc.
            // This would require additional functionality to be exposed in lib.rs

            println!("\nNote: This example demonstrates parsing a JSON document and");
            println!("analyzing its memory usage with the colchis crate.");
            println!("\nTo explore the structure of the JSON document further,");
            println!("additional methods would need to be exposed in the public API.");
        }
        Err(err) => {
            println!("Error parsing JSON: {:?}", err);
        }
    }

    Ok(())
}
