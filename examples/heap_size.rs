use colchis::Document;
use std::io;

fn main() -> io::Result<()> {
    // Sample JSON document as a string
    let sample_json = r#"{
        "name": "Sample Document",
        "type": "test",
        "version": 1.0,
        "active": true,
        "tags": ["json", "example", "heap"],
        "metadata": {
            "created_at": "2023-05-15",
            "author": "colchis"
        }
    }"#;

    // Parse the JSON document
    println!("Parsing sample JSON document...");
    let document = Document::parse(sample_json.as_bytes()).expect("Failed to parse JSON document");

    // Report heap size
    let heap_size = document.heap_size();
    println!("Document heap size: {} bytes", heap_size);

    Ok(())
}
