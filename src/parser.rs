use std::{
    io::Read,
    num::ParseFloatError,
    sync::atomic::{AtomicU64, Ordering},
};

use struson::reader::{JsonReader, JsonStreamReader, ReaderError, ValueType};
use vers_vecs::BitVec;

use crate::{
    document::Document, info::NodeType, structure::Structure, text::TextUsageBuilder,
    tree_builder::TreeBuilder, usage::UsageBuilder,
};

const TEXT_USAGE_BLOCK_SIZE: usize = 1024 * 1024; // 1 MiB
const TEXT_USAGE_CACHE_BLOCKS: usize = 10;

pub(crate) struct Parser<R: Read, B: UsageBuilder> {
    reader: JsonStreamReader<R>,
    builder: Builder<B>,
}

pub(crate) struct Builder<B: UsageBuilder> {
    pub(crate) tree_builder: TreeBuilder<B>,
    pub(crate) text_builder: TextUsageBuilder,
    pub(crate) numbers: Vec<f64>,
    pub(crate) booleans: BitVec,
}

impl<B: UsageBuilder> Builder<B> {
    pub(crate) fn new() -> Self {
        Self {
            tree_builder: TreeBuilder::new(),
            text_builder: TextUsageBuilder::new(TEXT_USAGE_BLOCK_SIZE, TEXT_USAGE_CACHE_BLOCKS),
            numbers: Vec::new(),
            booleans: BitVec::new(),
        }
    }

    pub(crate) fn display_heap_sizes(&self) {
        let tree_heap_size = self.tree_builder.heap_size();
        let text_heap_size = self.text_builder.heap_size();
        let uncompressed_text_size = self.text_builder.uncompressed_size();
        let numbers_heap_size = self.numbers.len() * std::mem::size_of::<f64>();
        let booleans_heap_size = self.booleans.heap_size();
        println!(
            "Tree: {:>15} ({:>6} Mb), Text: {:>15} ({:>6} Mb), Text orig: {:>15} ({:>6} Mb), Numbers: {:>15} ({:>6} Mb), Booleans: {:>15} ({:>6} Mb)",
            tree_heap_size,
            tree_heap_size / (1024 * 1024),
            text_heap_size,
            text_heap_size / (1024 * 1024),
            uncompressed_text_size,
            uncompressed_text_size / (1024 * 1024),
            numbers_heap_size,
            numbers_heap_size / (1024 * 1024),
            booleans_heap_size,
            booleans_heap_size / (1024 * 1024)
        );
    }
}

#[derive(Debug)]
pub enum JsonParseError {
    Reader(ReaderError),
    NumberParseError(ParseFloatError),
}

impl From<ReaderError> for JsonParseError {
    fn from(err: ReaderError) -> Self {
        JsonParseError::Reader(err)
    }
}

impl From<ParseFloatError> for JsonParseError {
    fn from(err: ParseFloatError) -> Self {
        JsonParseError::NumberParseError(err)
    }
}

static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn parse<R: Read, B: UsageBuilder>(
    json: R,
) -> Result<Document<B::Index>, JsonParseError> {
    let parser = Parser::<R, B>::new(json);
    parser.parse()
}

impl<R: Read, B: UsageBuilder> Parser<R, B> {
    fn new(json: R) -> Self {
        Self {
            reader: JsonStreamReader::new(json),
            builder: Builder::new(),
        }
    }

    fn parse(mut self) -> Result<Document<B::Index>, JsonParseError> {
        self.parse_item()?;
        // both the positions and the text is compressed at this point.

        // now uncompress the position data and turn it into a succinct structure
        // This will use some memory per node type, which is then compacted down
        // into a succinct structure
        let structure = Structure::<B::Index>::new(self.builder.tree_builder);
        // finally complete the text usage
        let text_usage = self.builder.text_builder.build();
        Ok(Document::new(
            structure,
            text_usage,
            self.builder.numbers,
            self.builder.booleans,
        ))
    }

    fn parse_item(&mut self) -> Result<(), JsonParseError> {
        TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
        if TICK_COUNTER.load(Ordering::Relaxed) % 1000000 == 0 {
            // self.builder.tree_builder.display_heap_sizes();

            self.builder.display_heap_sizes();
        }
        match self.reader.peek()? {
            ValueType::Array => {
                self.reader.begin_array()?;
                self.builder.tree_builder.open(NodeType::Array);
                while self.reader.has_next()? {
                    self.parse_item()?;
                }
                self.reader.end_array()?;
                self.builder.tree_builder.close(NodeType::Array);
            }
            ValueType::Object => {
                self.reader.begin_object()?;
                self.builder.tree_builder.open(NodeType::Object);
                while self.reader.has_next()? {
                    let key = self.reader.next_name()?;
                    let close_field_id = self.builder.tree_builder.open_field(key);
                    self.parse_item()?;
                    self.builder.tree_builder.close_field(close_field_id);
                }
                self.reader.end_object()?;
                self.builder.tree_builder.close(NodeType::Object);
            }
            ValueType::String => {
                let str = self.reader.next_str()?;
                self.builder.tree_builder.open(NodeType::String);
                let _text_id = self.builder.text_builder.add_string(str);
                self.builder.tree_builder.close(NodeType::String);
            }
            ValueType::Number => {
                let number = self.reader.next_number()??;
                self.builder.tree_builder.open(NodeType::Number);
                self.builder.numbers.push(number);
                self.builder.tree_builder.close(NodeType::Number);
            }
            ValueType::Boolean => {
                let boolean = self.reader.next_bool()?;
                self.builder.tree_builder.open(NodeType::Boolean);
                self.builder.booleans.append(boolean);
                self.builder.tree_builder.close(NodeType::Boolean);
            }
            ValueType::Null => {
                self.reader.next_null()?;
                self.builder.tree_builder.open(NodeType::Null);
                self.builder.tree_builder.close(NodeType::Null);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struson_single_number() {
        let json = "42";
        let mut reader = JsonStreamReader::new(json.as_bytes());
        let nr: f64 = reader.next_number().unwrap().unwrap();
        assert_eq!(nr, 42f64);
    }
}
