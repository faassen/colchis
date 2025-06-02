use std::io::Write;
use std::ops::Range;

use flate2::write::{ZlibDecoder, ZlibEncoder};
use vers_vecs::SparseRSVec;

pub(crate) struct TextBuilder {
    encoded_string: ZlibEncoder<Vec<u8>>,
    total: usize,
    positions: Vec<u64>,
}

impl TextBuilder {
    pub(crate) fn new() -> Self {
        Self {
            encoded_string: ZlibEncoder::new(Vec::new(), flate2::Compression::fast()),
            positions: Vec::new(),
            total: 0,
        }
    }

    pub(crate) fn heap_size(&self) -> usize {
        self.encoded_string.total_out() as usize + self.positions.len() * std::mem::size_of::<u64>()
    }

    pub(crate) fn string_node(&mut self, text: &str) {
        let l = text.len();
        self.encoded_string.write_all(text.as_bytes()).unwrap();
        // terminator $, the 0 byte
        self.total += l;
        let position = self.total as u64;
        self.positions.push(position);
        self.encoded_string.write_all(b"\0").unwrap();
        self.total += 1; // for the terminator
    }

    pub(crate) fn build(self) -> TextUsage {
        let compressed = self
            .encoded_string
            .finish()
            .expect("Failed to finish encoding");
        let writer: Vec<u8> = Vec::new();
        let mut decoder = ZlibDecoder::new(writer);
        decoder.write_all(&compressed).unwrap();
        let writer = decoder.finish().unwrap();
        let s = String::from_utf8(writer).expect("Failed to decode zlib compressed string");
        println!("Decompressed: {}", s);
        TextUsage {
            sarray: SparseRSVec::new(&self.positions, self.total as u64),
            text: s,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct TextId(usize);

impl TextId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct TextUsage {
    text: String,
    sarray: SparseRSVec,
}

impl TextUsage {
    pub(crate) fn heap_size(&self) -> usize {
        self.text.len() + self.sarray.heap_size()
    }

    #[allow(dead_code)]
    pub(crate) fn text_id(&self, index: usize) -> TextId {
        TextId(self.sarray.rank1(index as u64) as usize)
    }

    pub(crate) fn text_index(&self, text_id: TextId) -> usize {
        let id = text_id.0;
        if id == 0 {
            0
        } else {
            // we add 1 here as we want the index of the actual start of the
            // text rather than the terminator
            // unwrap is okay as we know we have a text id already
            self.sarray.select1(id - 1) as usize + 1
        }
    }

    pub(crate) fn text_range(&self, text_id: TextId) -> Range<usize> {
        let start = self.text_index(text_id);
        let end = self.text_index(TextId(text_id.0 + 1));
        start..(end - 1)
    }

    pub(crate) fn text_value(&self, text_id: TextId) -> &str {
        let range = self.text_range(text_id);
        &self.text[range]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_text_beginning() {
        let mut builder = TextBuilder::new();
        builder.string_node("hello");
        let usage = builder.build();
        let text_id = usage.text_id(0);
        assert_eq!(usage.text_index(text_id), 0);
    }

    #[test]
    fn test_one_text_middle() {
        let mut builder = TextBuilder::new();
        builder.string_node("hello");
        let usage = builder.build();
        let text_id = usage.text_id(3);
        assert_eq!(usage.text_index(text_id), 0);
    }

    #[test]
    fn test_two_texts() {
        let mut builder = TextBuilder::new();
        // 0..5
        builder.string_node("hello");
        // 6..11
        builder.string_node("world");
        let usage = builder.build();

        // in 'hello' text
        let text_id = usage.text_id(0);
        assert_eq!(usage.text_index(text_id), 0);
        let text_id = usage.text_id(1);
        assert_eq!(usage.text_index(text_id), 0);

        // in 'world' text
        let text_id = usage.text_id(6);
        assert_eq!(usage.text_index(text_id), 6);
        let text_id = usage.text_id(8);
        assert_eq!(usage.text_index(text_id), 6);
    }

    #[test]
    fn test_two_texts_range() {
        let mut builder = TextBuilder::new();
        // 0..5
        builder.string_node("hello");
        // 6..11
        builder.string_node("world");
        let usage = builder.build();

        assert_eq!(usage.text_range(TextId(0)), 0..5);
        assert_eq!(usage.text_range(TextId(1)), 6..11);
    }

    #[test]
    fn test_two_texts_value() {
        let mut builder = TextBuilder::new();
        builder.string_node("hello");
        builder.string_node("world");
        let usage = builder.build();

        assert_eq!(usage.text_value(TextId(0)), "hello");
        assert_eq!(usage.text_value(TextId(1)), "world");
    }
}
