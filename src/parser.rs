use std::{
    io::Read,
    num::ParseFloatError,
    sync::atomic::{AtomicU64, Ordering},
};

use struson::reader::{JsonReader, JsonStreamReader, ReaderError, ValueType};
use vers_vecs::BitVec;

use crate::{
    document::Document,
    info::NodeType,
    structure::Structure,
    text_usage::TextBuilder,
    tree_builder::TreeBuilder,
    usage::{EliasFanoUsageIndex, RoaringUsageBuilder},
};

pub(crate) struct Parser<R: Read> {
    reader: JsonStreamReader<R>,
    builder: Builder,
}

pub(crate) struct Builder {
    pub(crate) tree_builder: TreeBuilder<RoaringUsageBuilder>,
    pub(crate) text_builder: TextBuilder,
    pub(crate) numbers: Vec<f64>,
    pub(crate) booleans: BitVec,
}

impl Builder {
    pub(crate) fn new() -> Self {
        Self {
            tree_builder: TreeBuilder::new(),
            text_builder: TextBuilder::new(),
            numbers: Vec::new(),
            booleans: BitVec::new(),
        }
    }

    pub(crate) fn display_heap_sizes(&self) {
        let tree_heap_size = self.tree_builder.heap_size();
        let text_heap_size = self.text_builder.heap_size();
        println!(
            "Tree heap size: {:>15} ({:>6} Mb), Text heap size: {:>15} ({:>6} Mb)",
            tree_heap_size,
            tree_heap_size / 1_000_000,
            text_heap_size,
            text_heap_size / 1_000_000
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

impl<R: Read> Parser<R> {
    pub(crate) fn new(json: R) -> Self {
        Self {
            reader: JsonStreamReader::new(json),
            builder: Builder::new(),
        }
    }

    pub(crate) fn parse(mut self) -> Result<Document, JsonParseError> {
        self.parse_item()?;
        let structure = Structure::<EliasFanoUsageIndex>::new(self.builder.tree_builder);
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
            self.builder.tree_builder.display_heap_sizes();
            // self.builder.display_heap_sizes();
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
                    let key = self.reader.next_name_owned()?;
                    let node_type = NodeType::Field(key);
                    // TODO: we could do away with the clone if we used a cow perhaps
                    self.builder.tree_builder.open(node_type.clone());
                    self.parse_item()?;
                    self.builder.tree_builder.close(node_type);
                }
                self.reader.end_object()?;
                self.builder.tree_builder.close(NodeType::Object);
            }
            ValueType::String => {
                let str = self.reader.next_str()?;
                self.builder.tree_builder.open(NodeType::String);
                self.builder.text_builder.string_node(str);
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
