use colchis::text::TextUsageBuilder;

fn main() {
    // Create a compressed storage with custom settings
    let mut builder = TextUsageBuilder::new(1024, 5);

    // Add some strings to the storage
    let text1 = "Hello, world! This is a test string.";
    let text2 = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
    let text3 = "The quick brown fox jumps over the lazy dog.";
    let text4 = "Rust is a systems programming language focused on safety and performance.";

    println!("Adding strings to compressed storage...");
    let id1 = builder.add_string(text1);
    let id2 = builder.add_string(text2);
    let id3 = builder.add_string(text3);
    let id4 = builder.add_string(text4);

    // Add a very long string that will likely be compressed well
    let long_text = "This is a very long string that repeats itself. ".repeat(100);
    let id5 = builder.add_string(&long_text);

    let usage = builder.build();

    // Retrieve and verify the strings
    println!("\nRetrieving strings from storage...");
    println!("Text 1: {}", usage.get_string(id1));
    println!("Text 2: {}", usage.get_string(id2));
    println!("Text 3: {}", usage.get_string(id3));
    println!("Text 4: {}", usage.get_string(id4));

    let retrieved_long = usage.get_string(id5);
    println!("Long text (first 100 chars): {}...", &retrieved_long[..100]);

    // Display storage statistics
    let stats = usage.stats();
    println!("\nStorage Statistics:");
    println!("Total texts: {}", stats.total_texts);
    println!("Total blocks: {}", stats.total_blocks);
    println!("Original size: {} bytes", stats.original_size);
    println!("Compressed size: {} bytes", stats.compressed_size);
    println!("Compression ratio: {:.2}%", stats.compression_ratio * 100.0);
    println!(
        "Space saved: {:.2}%",
        (1.0 - stats.compression_ratio) * 100.0
    );
}
