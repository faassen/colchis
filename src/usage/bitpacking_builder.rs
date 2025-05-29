use bitpacking::{BitPacker, BitPacker4x};
use vers_vecs::SparseRSVec;

use crate::{info::NodeInfoId, lookup::NodeLookup};

use super::{EliasFanoUsageIndex, UsageBuilder};

#[derive(Clone)]
struct BlockInfo {
    // the initial value for this block
    initial_value: Option<u32>,
    num_bits: u8,
    compressed_start: usize,
    compressed_len: usize,
}

#[derive(Clone)]
struct Packed {
    compressed: Vec<u8>,
    remainder: Vec<u32>,
    block_infos: Vec<BlockInfo>,
    used: usize,
    // the initial value we are to use for the next block
    initial_value: Option<u32>,
}

impl Packed {
    fn new() -> Self {
        Self {
            compressed: Vec::new(),
            remainder: Vec::new(),
            block_infos: Vec::new(),
            used: 0,
            initial_value: None,
        }
    }

    fn heap_size(&self) -> usize {
        self.compressed.len() * std::mem::size_of::<u8>()
            + self.remainder.len() * std::mem::size_of::<u32>()
            + self.block_infos.len() * std::mem::size_of::<BlockInfo>()
    }

    fn append(&mut self, value: u32) {
        // add stuff to remainder until it reaches the block size
        self.remainder.push(value);
        if self.remainder.len() < BitPacker4x::BLOCK_LEN {
            return;
        }
        // now pack the remainder into a single block
        let bitpacker = BitPacker4x::new();

        let num_bits: u8 = bitpacker.num_bits_strictly_sorted(self.initial_value, &self.remainder);

        // we have reached the block size, so we can compress the
        // next block. first reserve enough space in compressed
        let compressed_start = self.used;
        self.compressed
            .resize(compressed_start + 4 * BitPacker4x::BLOCK_LEN, 0);
        let compressed_len = bitpacker.compress_strictly_sorted(
            self.initial_value,
            &self.remainder,
            &mut self.compressed[compressed_start..],
            num_bits,
        );
        self.block_infos.push(BlockInfo {
            initial_value: self.initial_value,
            compressed_start,
            compressed_len,
            num_bits,
        });
        // we now determine how much packed space we actually used
        self.used += compressed_len;
        // update the initial value to the last bit of the remainer
        self.initial_value = self.remainder.last().cloned();
        // now we can clear the remainder
        self.remainder.clear();
    }

    fn decompressed(&self) -> Vec<u32> {
        let mut decompressed_data = Vec::new();
        for block_info in &self.block_infos {
            // add enough space for another block
            let decompressed_start = decompressed_data.len();
            decompressed_data.resize(decompressed_start + BitPacker4x::BLOCK_LEN, 0);
            let bitpacker = BitPacker4x::new();
            bitpacker.decompress_strictly_sorted(
                block_info.initial_value,
                &self.compressed[block_info.compressed_start
                    ..block_info.compressed_start + block_info.compressed_len],
                &mut decompressed_data[decompressed_start..],
                block_info.num_bits,
            );
        }
        // now we add the remainder
        decompressed_data.extend(&self.remainder);
        decompressed_data
    }
}

pub struct BitpackingUsageBuilder {
    usage: Vec<Packed>,
    node_lookup: NodeLookup,
    len: usize,
}

impl UsageBuilder for BitpackingUsageBuilder {
    type Index = EliasFanoUsageIndex;

    fn new() -> Self {
        Self {
            usage: Vec::new(),
            node_lookup: NodeLookup::new(),
            len: 0,
        }
    }

    fn heap_size(&self) -> usize {
        let usage_heap_size: usize = self.usage.iter().map(|packed| packed.heap_size()).sum();
        self.node_lookup.heap_size() + usage_heap_size
    }

    fn node_lookup_mut(&mut self) -> &mut NodeLookup {
        &mut self.node_lookup
    }

    fn append(&mut self, node_info_id: NodeInfoId) {
        // get the positions for this node_info_id; make it an empty vec if it doesn't exist yet
        let i = node_info_id.id() as usize;
        if self.usage.len() <= i {
            self.usage.resize(i + 1, Packed::new());
        }
        let positions = self.usage.get_mut(i).expect("Entry should be present");
        positions.append(self.len as u32);
        self.len += 1;
    }

    fn build(mut self) -> Self::Index {
        let mut sparse_rs_vecs = Vec::with_capacity(self.node_lookup.len());
        // drain usage so we can throw away memory early
        for packed in self.usage.drain(..) {
            let positions = packed
                .decompressed()
                .into_iter()
                .map(|i| i as u64)
                .collect::<Vec<_>>();
            let sparse_rs_vec = SparseRSVec::new(&positions, self.len as u64);
            sparse_rs_vecs.push(sparse_rs_vec);
        }
        Self::Index::new(sparse_rs_vecs, self.node_lookup, self.len)
    }
}

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
        let size = 10000usize;
        let mut initial_data = Vec::new();
        for i in 0..size {
            initial_data.push((i * 2) as u32);
        }
        let mut packed = Packed::new();
        for value in &initial_data {
            packed.append(*value);
        }
        let original_size = size * 4;
        let compressed_size = packed.compressed.len();
        let difference = original_size - compressed_size;
        println!(
            "Compressed {} values, original size: {}, compressed size: {}, difference: {}",
            size, original_size, compressed_size, difference
        );
        let decompressed_data = packed.decompressed();
        assert_eq!(initial_data, decompressed_data)
    }
}
