use std::cell::RefCell;
use std::io::{Read, Write};

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
    start: usize,
    length: usize,
}

#[derive(Debug)]
struct Block {
    compressed_data: Vec<u8>,
    original_size: usize,
}

impl Block {
    fn compress(data: &[u8]) -> Self {
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
    current_block_texts: Vec<(usize, usize)>,
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
        self.current_block_texts.push((start, text_bytes.len()));

        text_id
    }

    fn finalize_current_block(&mut self) -> () {
        if self.current_block_texts.is_empty() {
            // nothing to finalize, just return
            return;
        }

        let block_id = BlockId::new(self.blocks.len());

        // Now we need to create a text info for each text in this block
        for (start, length) in &self.current_block_texts {
            let text_info = TextInfo {
                block_id,
                start: *start,
                length: *length,
            };
            self.text_infos.push(text_info);
        }
        // Create compressed block
        let block = Block::compress(&self.current_block_buffer);

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
    cache: RefCell<LruCache<BlockId, Vec<u8>>>,
}

impl TextUsage {
    fn new(cache_capacity: usize, blocks: Vec<Block>, text_infos: Vec<TextInfo>) -> Self {
        Self {
            blocks,
            text_infos,
            cache: RefCell::new(LruCache::new(cache_capacity.try_into().unwrap())),
        }
    }

    fn string_data(text_info: &TextInfo, block_data: &[u8]) -> String {
        let text_bytes = &block_data[text_info.start..text_info.start + text_info.length];
        // TODO: could do an unchecked conversion here as we should be sure
        // it's valid UTF-8
        String::from_utf8_lossy(text_bytes).into_owned()
    }

    /// Retrieve a string by its TextId
    pub fn get_string(&self, text_id: TextId) -> String {
        let text_info = self.text_infos.get(text_id.0).expect("TextId should exist");
        // first look for block in LRU cache
        let mut cache = self.cache.borrow_mut();
        let block_data = cache.get(&text_info.block_id);
        if let Some(uncompressed_data) = block_data {
            return Self::string_data(text_info, uncompressed_data);
        }

        // okay it was not in the cache, so we need to decompress the block
        let block = self
            .blocks
            .get(text_info.block_id.as_index())
            .expect("Compressed block should exist");
        let block_data = block.decompress();
        let s = Self::string_data(text_info, &block_data);
        // now we can add the decompressed block to the cache
        cache.put(text_info.block_id, block_data);
        s
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
            cache_size: self.cache.borrow().len(),
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
        assert_eq!(retrieved, text);
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
            assert_eq!(retrieved, texts[i]);
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

        assert_eq!(usage.get_string(id1), long_text);
        assert_eq!(usage.get_string(id2), short_text);
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

        assert_eq!(usage.get_string(id1), long_text);
        assert_eq!(usage.stats().cache_size, 1);

        assert_eq!(usage.get_string(id2), short_text);
        assert_eq!(usage.stats().cache_size, 2);
    }

    #[test]
    fn test_empty_string() {
        let mut builder = TextUsageBuilder::new(1000, 5);

        let text_id = builder.add_string("");

        let usage = builder.build();
        // Retrieve the empty string
        let retrieved = usage.get_string(text_id);
        assert_eq!(retrieved, "");
        assert_eq!(usage.stats().total_texts, 1);
    }
}
