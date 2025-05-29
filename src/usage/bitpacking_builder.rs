use bitpacking::{BitPacker, BitPacker4x};

use crate::{info::NodeInfoId, lookup::NodeLookup};

use super::{EliasFanoUsageIndex, UsageBuilder};

#[derive(Clone)]
struct Packed {
    compressed: Vec<u8>,
    remainder: Vec<u32>,
    used: usize,
    last_initial_value: Option<u32>,
}

impl Packed {
    fn new() -> Self {
        Self {
            compressed: Vec::new(),
            remainder: Vec::new(),
            used: 0,
            last_initial_value: Some(0),
        }
    }

    fn heap_size(&self) -> usize {
        self.compressed.len() * std::mem::size_of::<u8>()
            + self.remainder.len() * std::mem::size_of::<u32>()
    }

    fn append(&mut self, value: u32) {
        // add stuff to remainder until it reaches the block size
        self.remainder.push(value);
        if self.remainder.len() < BitPacker4x::BLOCK_LEN {
            return;
        }
        // now pack the remainder into a single block
        let bitpacker = BitPacker4x::new();
        let num_bits: u8 =
            bitpacker.num_bits_strictly_sorted(self.last_initial_value, &self.remainder);

        // we have reached the block size, so we can compress the
        // next block. first reserve enough space in compressed
        let compressed_start = self.used;
        self.compressed
            .resize(compressed_start + 4 * BitPacker4x::BLOCK_LEN, 0);
        let compressed_len = bitpacker.compress_strictly_sorted(
            self.last_initial_value,
            &self.remainder,
            &mut self.compressed[compressed_start..],
            num_bits,
        );
        // we now determine how much packed space we actually used
        self.used += compressed_len;
        // new initial value is the last value in the remainder
        self.last_initial_value = self.remainder.last().cloned();
        // now we can clear the remainder
        self.remainder.clear();
    }
}

pub struct BitpackingBuilder {
    usage: Vec<Packed>,
    node_lookup: NodeLookup,
    len: usize,
}

// impl UsageBuilder for BitpackingBuilder {
//     type Index = EliasFanoUsageIndex;

//     fn new() -> Self {
//         Self {
//             usage: Vec::new(),
//             node_lookup: NodeLookup::new(),
//             len: 0,
//         }
//     }

//     fn heap_size(&self) -> usize {
//         let usage_heap_size: usize = self.usage.iter().map(|packed| packed.heap_size()).sum();
//         self.node_lookup.heap_size() + usage_heap_size
//     }

//     fn node_lookup_mut(&mut self) -> &mut NodeLookup {
//         &mut self.node_lookup
//     }

//     fn append(&mut self, node_info_id: NodeInfoId) {
//         // get the positions for this node_info_id; make it an empty vec if it doesn't exist yet
//         let i = node_info_id.id() as usize;
//         if self.usage.len() <= i {
//             self.usage.resize(i + 1, Packed::new());
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packing() {
        let initial_value = 0u32;

        let mut my_data = vec![0u32; BitPacker4x::BLOCK_LEN];

        for i in 0..BitPacker4x::BLOCK_LEN {
            my_data[i] = initial_value + (i * 2) as u32;
        }

        let bitpacker = BitPacker4x::new();
        let num_bits: u8 = bitpacker.num_bits_strictly_sorted(Some(initial_value), &my_data);
        // println!("num bits: {}", num_bits);
        let mut compressed = vec![0u8; 4 * BitPacker4x::BLOCK_LEN];

        let compressed_len =
            bitpacker.compress_strictly_sorted(None, &my_data, &mut compressed[..], num_bits);

        let mut decompressed = vec![0u32; BitPacker4x::BLOCK_LEN];

        bitpacker.decompress_strictly_sorted(
            None,
            &compressed[..compressed_len],
            &mut decompressed[..],
            num_bits,
        );

        // println!("compressed len: {}", compressed_len);

        assert_eq!(&my_data, &decompressed);
    }

    #[test]
    fn test_packed() {
        let mut packed = Packed::new();
        let size = 10000usize;
        for i in 0..size {
            packed.append((i * 2) as u32);
        }
        let original_size = size * 4;
        let compressed_size = packed.compressed.len();
        let difference = original_size - compressed_size;
        println!(
            "Compressed {} values, original size: {}, compressed size: {}, difference: {}",
            size, original_size, compressed_size, difference
        );
    }
}
