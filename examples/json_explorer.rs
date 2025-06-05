use colchis::{BitpackingUsageBuilder, Document};
use std::env;
use std::fs::File;
use std::io;
use tikv_jemalloc_ctl::{epoch, stats};

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

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
    // get file size in bytes
    let file_size = std::fs::metadata(file_path)?.len() as usize;
    let file = File::open(file_path)?;
    // do not use a buffer, get a reader to avoid unnecessary memory usage
    // no need for a bufreader as struson handles buffering internally
    match Document::parse::<BitpackingUsageBuilder, _>(&file) {
        Ok(document) => {
            // advance the epoch to ensure jemalloc stats are up-to-date
            epoch::advance().unwrap();

            let allocated = stats::allocated::read().unwrap();
            let resident = stats::resident::read().unwrap();
            println!("\n===== Memory usage =====");

            println!(
                "Original file size: {} ({:.4} Mb)",
                file_size,
                to_mb(file_size)
            );
            println!(
                "Allocated: {} bytes ({:.4} Mb), Resident: {} bytes ({:.4} Mb)",
                allocated,
                to_mb(allocated),
                resident,
                to_mb(resident)
            );
            // Display document information
            let heap_size = document.heap_size();
            println!(
                "Heap size: {} bytes ({:.4} Mb)",
                heap_size,
                to_mb(heap_size)
            );
            let more_than_file_size = resident - file_size;
            let less_than_file_size = file_size - heap_size;
            let percentage_over_file_size = if file_size > 0 {
                (more_than_file_size as f64 / file_size as f64) * 100.0
            } else {
                0.0
            };
            let percentage_under_file_size = if file_size > 0 {
                (less_than_file_size as f64 / file_size as f64) * 100.0
            } else {
                0.0
            };
            println!(
                "Resident over file size: {} ({:.4} Mb), Final under file size: {} ({:.4} Mb)",
                more_than_file_size,
                to_mb(more_than_file_size),
                less_than_file_size,
                to_mb(less_than_file_size)
            );
            println!(
                "Resident usage is {:.2}% more than file size, finally allocated is {:.2}% less than file size",
                percentage_over_file_size, percentage_under_file_size
            );
        }
        Err(err) => {
            println!("Error parsing JSON: {:?}", err);
        }
    }

    Ok(())
}

fn to_mb(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
