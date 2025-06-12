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
            println!("\n===== Size comparisons =====");
            compare_sizes("Resident memory", resident, "File size", file_size);
            compare_sizes("Heap size", heap_size, "File size", file_size);
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

fn compare_sizes(name1: &str, size1: usize, name2: &str, size2: usize) {
    if size1 > size2 {
        let difference = size1 - size2;
        let percentage = if size2 > 0 {
            (difference as f64 / size2 as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "{} is {} bytes ({:.4} Mb) larger than {} ({:.2}% increase)",
            name1,
            difference,
            to_mb(difference),
            name2,
            percentage
        );
    } else if size1 < size2 {
        let difference = size2 - size1;
        let percentage = if size2 > 0 {
            (difference as f64 / size2 as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "{} is {} bytes ({:.4} Mb) smaller than {} ({:.2}% decrease)",
            name1,
            difference,
            to_mb(difference),
            name2,
            percentage
        );
    } else {
        println!("{} and {} are equal in size", name1, name2);
    }
}
