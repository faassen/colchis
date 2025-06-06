use std::cell::RefCell;
use std::io::{Read, Write};
use std::num::NonZeroUsize;
use std::ops::Range;
use std::sync::Arc;

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use lru::LruCache;

/// Unique identifier for stored text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextId(usize);

impl TextId {
    fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Unique identifier for a compressed block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BlockId(usize);

impl BlockId {
    fn new(id: usize) -> Self {
        Self(id)
    }

    fn as_index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
struct TextInfo {
    block_id: BlockId,
}

#[derive(Debug)]
struct Block {
    compressed_data: Vec<u8>,
    original_size: usize,
    // the start text id for this block
    start_text_id: TextId,
    // the ranges of text ids in this block
    ranges: Vec<Range<usize>>,
}

impl Block {
    fn compress(start_text_id: TextId, ranges: Vec<Range<usize>>, data: &[u8]) -> Self {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .expect("Memory write should not result in IO error");
        let compressed_data = encoder
            .finish()
            .expect("Memory write should not result in IO error");

        Block {
            compressed_data,
            original_size: data.len(),
            start_text_id,
            ranges,
        }
    }

    fn decompress(&self) -> Vec<u8> {
        let mut decoder = DeflateDecoder::new(self.compressed_data.as_slice());
        let mut decompressed = Vec::with_capacity(self.original_size);
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    }
}

/// Builder for creating compressed string storage
pub struct TextUsageBuilder {
    block_size: usize,
    cache_capacity: usize,
    current_block_buffer: Vec<u8>,
    current_block_texts: Vec<Range<usize>>,
    blocks: Vec<Block>,
    text_infos: Vec<TextInfo>,
}

impl TextUsageBuilder {
    pub fn new(block_size: usize, cache_capacity: usize) -> Self {
        Self {
            block_size,
            cache_capacity,
            blocks: Vec::new(),
            text_infos: Vec::new(),
            current_block_buffer: Vec::new(),
            current_block_texts: Vec::new(),
        }
    }

    /// Add a string to the storage and return its TextId
    pub fn add_string(&mut self, text: &str) -> TextId {
        let text_bytes = text.as_bytes();
        // we use the length of the previously compressed texts plus the ones
        // we are currently building to determine a unique incremental text id
        let text_id = TextId::new(self.text_infos.len() + self.current_block_texts.len());

        // Check if adding this text would exceed block size
        if (self.current_block_buffer.len() + text_bytes.len()) > self.block_size
            // if this is an empty block already, we are going to add the text string to that
            && !self.current_block_buffer.is_empty()
        {
            // finalize the current block and make a new block ready for new text
            self.finalize_current_block();
        }

        let start = self.current_block_buffer.len();
        self.current_block_buffer.extend_from_slice(text_bytes);

        // track that we've added this text to the current block
        self.current_block_texts
            .push(start..start + text_bytes.len());

        text_id
    }

    fn finalize_current_block(&mut self) -> () {
        if self.current_block_texts.is_empty() {
            // nothing to finalize, just return
            return;
        }

        let block_id = BlockId::new(self.blocks.len());

        // Now we need to create a text info for each text in this block
        let start_text_id = TextId::new(self.text_infos.len());
        for _ in &self.current_block_texts {
            let text_info = TextInfo { block_id };
            self.text_infos.push(text_info);
        }
        // Create compressed block
        let block = Block::compress(
            start_text_id,
            self.current_block_texts.clone(),
            &self.current_block_buffer,
        );

        self.blocks.push(block);

        // Clear current block
        self.current_block_buffer.clear();
        self.current_block_texts.clear();
    }

    pub fn build(mut self) -> TextUsage {
        // if there is a half-finished block, finalize it
        self.finalize_current_block();
        TextUsage::new(self.cache_capacity, self.blocks, self.text_infos)
    }
}

/// Main compressed string storage structure
pub struct TextUsage {
    blocks: Vec<Block>,
    text_infos: Vec<TextInfo>,
    cache: RefCell<LruCache<BlockId, Arc<[Arc<str>]>>>,
    cache_capacity: usize,
}

impl TextUsage {
    fn new(cache_capacity: usize, blocks: Vec<Block>, text_infos: Vec<TextInfo>) -> Self {
        // LruCache requires NonZeroUsize, so we use 1 as minimum capacity
        let capacity = NonZeroUsize::new(cache_capacity.max(1)).unwrap();
        Self {
            blocks,
            text_infos,
            cache: RefCell::new(LruCache::new(capacity)),
            cache_capacity,
        }
    }

    // extract all strings from the block data
    fn block_slices(&self, block_id: BlockId, block_data: Vec<u8>) -> Vec<Arc<str>> {
        let block = self
            .blocks
            .get(block_id.as_index())
            .expect("BlockId should exist");
        block
            .ranges
            .iter()
            .map(|range| {
                // SAFETY: we can do an unchecked conversion here as we know the data is valid UTF-8
                let s = unsafe { std::str::from_utf8_unchecked(&block_data[range.clone()]) };
                // this is not zero-copy but we'll accept that
                Arc::from(s)
            })
            .collect()
    }

    /// Retrieve a string by its TextId
    pub fn get_string(&self, text_id: TextId) -> Arc<str> {
        let text_info = self.text_infos.get(text_id.0).expect("TextId should exist");

        let block = self
            .blocks
            .get(text_info.block_id.as_index())
            .expect("Block should exist");

        let block_slices = {
            if self.cache_capacity > 0 {
                let mut cache = self.cache.borrow_mut();
                if let Some(cached) = cache.get(&text_info.block_id) {
                    cached.clone()
                } else {
                    // Decompress and cache
                    let block_data = block.decompress();
                    let block_slices: Arc<[Arc<str>]> =
                        Arc::from(self.block_slices(text_info.block_id, block_data));
                    cache.put(text_info.block_id, block_slices.clone());
                    block_slices
                }
            } else {
                let block_data = block.decompress();
                Arc::from(self.block_slices(text_info.block_id, block_data))
            }
        };

        let offset = text_id.0 - block.start_text_id.0;
        block_slices[offset].clone()
    }

    /// Get storage statistics
    pub fn stats(&self) -> StorageStats {
        let total_compressed_size: usize = self
            .blocks
            .iter()
            .map(|block| block.compressed_data.len())
            .sum();

        let total_original_size: usize = self
            .blocks
            .iter()
            .map(|block| block.original_size)
            .sum::<usize>();

        StorageStats {
            total_texts: self.text_infos.len(),
            total_blocks: self.blocks.len(),
            compressed_size: total_compressed_size,
            original_size: total_original_size,
            compression_ratio: if total_original_size > 0 {
                total_compressed_size as f64 / total_original_size as f64
            } else {
                0.0
            },
            cache_size: if self.cache_capacity == 0 {
                0
            } else {
                self.cache.borrow().len()
            },
        }
    }
}

/// Statistics about the compressed storage
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_texts: usize,
    pub total_blocks: usize,
    pub compressed_size: usize,
    pub original_size: usize,
    pub compression_ratio: f64,
    pub cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_string() {
        let mut builder = TextUsageBuilder::new(100, 1);

        let text = "Hello, world!";
        let text_id = builder.add_string(text);

        let usage = builder.build();

        let retrieved = usage.get_string(text_id);
        assert_eq!(retrieved, text.into());
    }

    #[test]
    fn test_multiple_strings_same_block() {
        let mut builder = TextUsageBuilder::new(1000, 1);

        let texts = vec!["First text", "Second text", "Third text"];
        let mut text_ids = Vec::new();

        for text in &texts {
            text_ids.push(builder.add_string(text));
        }

        let usage = builder.build();

        for (i, text_id) in text_ids.iter().enumerate() {
            let retrieved = usage.get_string(*text_id);
            assert_eq!(retrieved, texts[i].into());
        }
        assert_eq!(usage.stats().total_blocks, 1);
    }

    #[test]
    fn test_multiple_blocks() {
        // short block size of only 10b bytes to force compression
        let mut builder = TextUsageBuilder::new(10, 1);

        // add a text beyond 10 bytes; this will fit in one block and force a new block
        let long_text = "This is a long text that should exceed the block size.";
        let id1 = builder.add_string(&long_text);
        // this should be in a new block
        let short_text = "Short";
        let id2 = builder.add_string(short_text);

        let usage = builder.build();

        assert_eq!(usage.get_string(id1), long_text.into());
        assert_eq!(usage.get_string(id2), short_text.into());
        assert_eq!(usage.stats().total_blocks, 2);
    }

    #[test]
    fn test_cache_functionality() {
        // short block size of only 10b bytes to have multiple blocks, with a
        // generous cache size of 5 we should never even reach
        let mut builder = TextUsageBuilder::new(10, 5);

        // add a text beyond 10 bytes; this will fit in one block and force a new block
        let long_text = "This is a long text that should exceed the block size.";
        let id1 = builder.add_string(&long_text);
        // this should be in a new block
        let short_text = "Short";
        let id2 = builder.add_string(short_text);

        let usage = builder.build();

        assert_eq!(usage.get_string(id1), long_text.into());
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id2), short_text.into());
        assert_eq!(usage.stats().cache_size, 2);
    }

    #[test]
    fn test_empty_string() {
        let mut builder = TextUsageBuilder::new(1000, 5);

        let text_id = builder.add_string("");

        let usage = builder.build();
        // Retrieve the empty string
        let retrieved = usage.get_string(text_id);
        assert_eq!(retrieved, "".into());
        assert_eq!(usage.stats().total_texts, 1);
    }

    #[test]
    fn test_string_exactly_at_block_size() {
        let block_size = 20;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Create a string exactly 20 bytes long
        let exact_size_text = "12345678901234567890"; // exactly 20 bytes
        assert_eq!(exact_size_text.len(), block_size);

        let text_id = builder.add_string(exact_size_text);

        let usage = builder.build();
        let retrieved = usage.get_string(text_id);
        assert_eq!(retrieved, exact_size_text.into());
        assert_eq!(usage.stats().total_blocks, 1);
        assert_eq!(usage.stats().total_texts, 1);
    }

    #[test]
    fn test_string_exactly_fills_remaining_space() {
        let block_size = 20;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Add a string that partially fills the block
        let first_text = "Hello"; // 5 bytes
        let id1 = builder.add_string(first_text);

        // Add a string that exactly fills the remaining 15 bytes
        let second_text = "123456789012345"; // exactly 15 bytes
        assert_eq!(second_text.len(), 15);
        let id2 = builder.add_string(second_text);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), first_text.into());
        assert_eq!(usage.get_string(id2), second_text.into());
        assert_eq!(usage.stats().total_blocks, 1); // Should fit in one block
        assert_eq!(usage.stats().total_texts, 2);
    }

    #[test]
    fn test_string_one_byte_over_block_size() {
        let block_size = 20;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Add a string that partially fills the block
        let first_text = "Hello"; // 5 bytes
        let id1 = builder.add_string(first_text);

        // Add a string that would exceed block by 1 byte (16 bytes, but only 15 remaining)
        let second_text = "1234567890123456"; // 16 bytes
        assert_eq!(second_text.len(), 16);
        let id2 = builder.add_string(second_text);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), first_text.into());
        assert_eq!(usage.get_string(id2), second_text.into());
        assert_eq!(usage.stats().total_blocks, 2); // Should create two blocks
        assert_eq!(usage.stats().total_texts, 2);
    }

    #[test]
    fn test_massive_string_far_exceeding_block_size() {
        let block_size = 10;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Create a string that's 10x the block size
        let massive_text = "A".repeat(100); // 100 bytes, 10x block size
        let text_id = builder.add_string(&massive_text);

        let usage = builder.build();
        let retrieved = usage.get_string(text_id);
        assert_eq!(retrieved, massive_text.into());
        assert_eq!(usage.stats().total_blocks, 1); // Still one block, just large
        assert_eq!(usage.stats().total_texts, 1);
    }

    #[test]
    fn test_multiple_strings_cumulative_exact_block_size() {
        let block_size = 20;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Add strings that together sum to exactly block_size
        let text1 = "12345"; // 5 bytes
        let text2 = "67890"; // 5 bytes
        let text3 = "ABCDEFGHIJ"; // 10 bytes
        // Total: 20 bytes exactly

        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);
        let id3 = builder.add_string(text3);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.get_string(id3), text3.into());
        assert_eq!(usage.stats().total_blocks, 1);
        assert_eq!(usage.stats().total_texts, 3);
    }

    #[test]
    fn test_sequential_block_boundary_crossings() {
        let block_size = 10;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Each string will force a new block
        let texts = vec![
            "1234567890A", // 11 bytes - exceeds block
            "BCDEFGHIJK",  // 10 bytes - exactly block size
            "LMNOPQRSTU",  // 10 bytes - exactly block size
        ];

        let mut text_ids = Vec::new();
        for text in &texts {
            text_ids.push(builder.add_string(text));
        }

        let usage = builder.build();

        for (i, text_id) in text_ids.iter().enumerate() {
            assert_eq!(usage.get_string(*text_id), texts[i].into());
        }
        assert_eq!(usage.stats().total_blocks, 3); // Each string in its own block
        assert_eq!(usage.stats().total_texts, 3);
    }

    #[test]
    fn test_empty_string_at_block_boundary() {
        let block_size = 10;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Fill block exactly
        let full_text = "1234567890"; // exactly 10 bytes
        let id1 = builder.add_string(full_text);

        // Add empty string - should go to new block
        let id2 = builder.add_string("");

        // Add another string to same block as empty string
        let next_text = "Hello";
        let id3 = builder.add_string(next_text);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), full_text.into());
        assert_eq!(usage.get_string(id2), "".into());
        assert_eq!(usage.get_string(id3), next_text.into());
        assert_eq!(usage.stats().total_blocks, 2);
        assert_eq!(usage.stats().total_texts, 3);
    }

    #[test]
    fn test_zero_size_block_limit() {
        // Edge case: what happens with block_size of 0?
        // This should still work - every string gets its own block
        let mut builder = TextUsageBuilder::new(0, 5);

        let text1 = "A";
        let text2 = "B";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.stats().total_blocks, 2); // Each string in its own block
        assert_eq!(usage.stats().total_texts, 2);
    }

    #[test]
    fn test_alternating_exact_and_overflow() {
        let block_size = 10;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Pattern: exact fit, then overflow, repeat
        let exact_fit = "1234567890"; // exactly 10 bytes
        let overflow = "12345678901"; // 11 bytes

        let id1 = builder.add_string(exact_fit); // Block 1: exactly fits
        let id2 = builder.add_string(overflow); // Block 2: overflows
        let id3 = builder.add_string(exact_fit); // Block 3: exactly fits
        let id4 = builder.add_string(overflow); // Block 4: overflows

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), exact_fit.into());
        assert_eq!(usage.get_string(id2), overflow.into());
        assert_eq!(usage.get_string(id3), exact_fit.into());
        assert_eq!(usage.get_string(id4), overflow.into());
        assert_eq!(usage.stats().total_blocks, 4);
        assert_eq!(usage.stats().total_texts, 4);
    }

    #[test]
    fn test_multiple_empty_strings_at_boundaries() {
        let block_size = 5;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Fill first block exactly
        let text1 = "12345"; // exactly 5 bytes
        let id1 = builder.add_string(text1);

        // Add multiple empty strings - should all go to next block
        let id2 = builder.add_string("");
        let id3 = builder.add_string("");
        let id4 = builder.add_string("");

        // Add regular string to same block
        let text2 = "ABC";
        let id5 = builder.add_string(text2);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.get_string(id2), "".into());
        assert_eq!(usage.get_string(id3), "".into());
        assert_eq!(usage.get_string(id4), "".into());
        assert_eq!(usage.get_string(id5), text2.into());
        assert_eq!(usage.stats().total_blocks, 2);
        assert_eq!(usage.stats().total_texts, 5);
    }

    #[test]
    fn test_single_byte_strings_at_boundary() {
        let block_size = 5;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Add exactly 5 single-byte strings to fill one block
        let id1 = builder.add_string("A");
        let id2 = builder.add_string("B");
        let id3 = builder.add_string("C");
        let id4 = builder.add_string("D");
        let id5 = builder.add_string("E");

        // Next string should go to new block
        let id6 = builder.add_string("F");

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), "A".into());
        assert_eq!(usage.get_string(id2), "B".into());
        assert_eq!(usage.get_string(id3), "C".into());
        assert_eq!(usage.get_string(id4), "D".into());
        assert_eq!(usage.get_string(id5), "E".into());
        assert_eq!(usage.get_string(id6), "F".into());
        assert_eq!(usage.stats().total_blocks, 2);
        assert_eq!(usage.stats().total_texts, 6);
    }

    #[test]
    fn test_block_size_one_byte() {
        // Extreme case: block size of 1 byte
        let mut builder = TextUsageBuilder::new(1, 5);

        let id1 = builder.add_string("A"); // 1 byte - fills block
        let id2 = builder.add_string("B"); // 1 byte - new block
        let id3 = builder.add_string("AB"); // 2 bytes - new block (exceeds)
        let id4 = builder.add_string(""); // 0 bytes - new block

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), "A".into());
        assert_eq!(usage.get_string(id2), "B".into());
        assert_eq!(usage.get_string(id3), "AB".into());
        assert_eq!(usage.get_string(id4), "".into());
        assert_eq!(usage.stats().total_blocks, 4);
        assert_eq!(usage.stats().total_texts, 4);
    }

    #[test]
    fn test_repeated_exact_block_fills() {
        let block_size = 10;
        let mut builder = TextUsageBuilder::new(block_size, 5);

        // Each string exactly fills a block
        let exact_text = "1234567890"; // exactly 10 bytes
        let id1 = builder.add_string(exact_text);
        let id2 = builder.add_string(exact_text);
        let id3 = builder.add_string(exact_text);

        let usage = builder.build();
        assert_eq!(usage.get_string(id1), exact_text.into());
        assert_eq!(usage.get_string(id2), exact_text.into());
        assert_eq!(usage.get_string(id3), exact_text.into());
        assert_eq!(usage.stats().total_blocks, 3); // Each in its own block
        assert_eq!(usage.stats().total_texts, 3);
    }

    #[test]
    fn test_cache_eviction_with_small_capacity() {
        let block_size = 10;
        let cache_capacity = 2; // Small cache that will cause eviction
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        // Create 4 blocks, but cache can only hold 2
        let text1 = "Block1Text"; // 10 bytes - Block 1
        let text2 = "Block2Text"; // 10 bytes - Block 2
        let text3 = "Block3Text"; // 10 bytes - Block 3
        let text4 = "Block4Text"; // 10 bytes - Block 4

        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);
        let id3 = builder.add_string(text3);
        let id4 = builder.add_string(text4);

        let usage = builder.build();

        // Access all strings - should cause cache eviction
        assert_eq!(usage.get_string(id1), text1.into()); // Cache: [Block1]
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id2), text2.into()); // Cache: [Block1, Block2]
        assert_eq!(usage.stats().cache_size, 2);

        assert_eq!(usage.get_string(id3), text3.into()); // Cache: [Block2, Block3] (Block1 evicted)
        assert_eq!(usage.stats().cache_size, 2);

        assert_eq!(usage.get_string(id4), text4.into()); // Cache: [Block3, Block4] (Block2 evicted)
        assert_eq!(usage.stats().cache_size, 2);

        // Access Block1 again - should require decompression
        assert_eq!(usage.get_string(id1), text1.into()); // Cache: [Block4, Block1] (Block3 evicted)
        assert_eq!(usage.stats().cache_size, 2);
    }

    #[test]
    fn test_zero_cache_capacity() {
        let block_size = 10;
        let cache_capacity = 0; // No caching
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        let text1 = "Block1Text";
        let text2 = "Block2Text";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);

        let usage = builder.build();

        // Every access should decompress fresh (no caching)
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.stats().cache_size, 0);

        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.stats().cache_size, 0);

        // Access again - still no caching
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.stats().cache_size, 0);
    }

    #[test]
    fn test_cache_thrashing_alternating_access() {
        let block_size = 10;
        let cache_capacity = 1; // Only 1 block can be cached
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        let text1 = "Block1Text";
        let text2 = "Block2Text";
        let text3 = "Block3Text";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);
        let id3 = builder.add_string(text3);

        let usage = builder.build();

        // Alternating access pattern that causes constant eviction
        assert_eq!(usage.get_string(id1), text1.into()); // Cache: [Block1]
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id2), text2.into()); // Cache: [Block2] (Block1 evicted)
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id1), text1.into()); // Cache: [Block1] (Block2 evicted)
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id3), text3.into()); // Cache: [Block3] (Block1 evicted)
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id2), text2.into()); // Cache: [Block2] (Block3 evicted)
        assert_eq!(usage.stats().cache_size, 1);
    }

    #[test]
    fn test_cache_hit_same_block_multiple_strings() {
        let block_size = 50;
        let cache_capacity = 2;
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        // Multiple strings in same block
        let text1 = "First";
        let text2 = "Second";
        let text3 = "Third";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);
        let id3 = builder.add_string(text3);

        let usage = builder.build();
        assert_eq!(usage.stats().total_blocks, 1); // All in same block

        // First access loads block into cache
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.stats().cache_size, 1);

        // Subsequent accesses to same block should be cache hits
        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.stats().cache_size, 1); // Still same block

        assert_eq!(usage.get_string(id3), text3.into());
        assert_eq!(usage.stats().cache_size, 1); // Still same block
    }

    #[test]
    fn test_cache_with_repeated_string_access() {
        let block_size = 10;
        let cache_capacity = 2;
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        let text1 = "Block1Text";
        let text2 = "Block2Text";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);

        let usage = builder.build();

        // Access same string multiple times
        for _ in 0..5 {
            assert_eq!(usage.get_string(id1), text1.into());
        }
        assert_eq!(usage.stats().cache_size, 1);

        // Access second string
        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.stats().cache_size, 2);

        // Access first string again - should be cache hit
        for _ in 0..3 {
            assert_eq!(usage.get_string(id1), text1.into());
        }
        assert_eq!(usage.stats().cache_size, 2);
    }

    #[test]
    fn test_cache_capacity_larger_than_blocks() {
        let block_size = 10;
        let cache_capacity = 10; // Much larger than number of blocks
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        let text1 = "Block1Text";
        let text2 = "Block2Text";
        let text3 = "Block3Text";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);
        let id3 = builder.add_string(text3);

        let usage = builder.build();
        assert_eq!(usage.stats().total_blocks, 3);

        // Access all blocks - should all fit in cache
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.get_string(id3), text3.into());
        assert_eq!(usage.stats().cache_size, 3);

        // Access in any order - should all be cache hits
        assert_eq!(usage.get_string(id3), text3.into());
        assert_eq!(usage.get_string(id1), text1.into());
        assert_eq!(usage.get_string(id2), text2.into());
        assert_eq!(usage.stats().cache_size, 3); // No eviction
    }

    #[test]
    fn test_cache_with_empty_strings() {
        let block_size = 10;
        let cache_capacity = 2;
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        let text1 = "Block1Text";
        let empty_id1 = builder.add_string(""); // Empty string in block 2
        let empty_id2 = builder.add_string(""); // Another empty string in block 2
        let text2 = "Block2Text"; // Regular text in block 2
        let text2_id = builder.add_string(text2); // This will be in block 2
        let id1 = builder.add_string(text1); // This will be in block 3

        let usage = builder.build();

        // Access strings from different blocks
        assert_eq!(usage.get_string(id1), text1.into()); // Block 3
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(empty_id1), "".into()); // Block 2
        assert_eq!(usage.stats().cache_size, 2);

        assert_eq!(usage.get_string(empty_id2), "".into()); // Block 2 (cache hit)
        assert_eq!(usage.stats().cache_size, 2);

        assert_eq!(usage.get_string(text2_id), text2.into()); // Block 2 (cache hit)
        assert_eq!(usage.stats().cache_size, 2);
    }

    #[test]
    fn test_cache_lru_ordering() {
        let block_size = 10;
        let cache_capacity = 3;
        let mut builder = TextUsageBuilder::new(block_size, cache_capacity);

        // Create 4 blocks
        let text1 = "Block1Text";
        let text2 = "Block2Text";
        let text3 = "Block3Text";
        let text4 = "Block4Text";
        let id1 = builder.add_string(text1);
        let id2 = builder.add_string(text2);
        let id3 = builder.add_string(text3);
        let id4 = builder.add_string(text4);

        let usage = builder.build();

        // Load first 3 blocks into cache
        assert_eq!(usage.get_string(id1), text1.into()); // Cache: [Block1]
        assert_eq!(usage.get_string(id2), text2.into()); // Cache: [Block1, Block2]
        assert_eq!(usage.get_string(id3), text3.into()); // Cache: [Block1, Block2, Block3]
        assert_eq!(usage.stats().cache_size, 3);

        // Access Block1 again to make it most recently used
        assert_eq!(usage.get_string(id1), text1.into()); // Cache: [Block2, Block3, Block1]
        assert_eq!(usage.stats().cache_size, 3);

        // Add Block4 - should evict Block2 (least recently used)
        assert_eq!(usage.get_string(id4), text4.into()); // Cache: [Block3, Block1, Block4]
        assert_eq!(usage.stats().cache_size, 3);

        // Access Block2 again - should require decompression
        assert_eq!(usage.get_string(id2), text2.into()); // Cache: [Block1, Block4, Block2]
        assert_eq!(usage.stats().cache_size, 3);
    }
}
