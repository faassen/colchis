use std::{io::Read, num::ParseFloatError};

use struson::reader::{JsonReader, JsonStreamReader, ReaderError, ValueType};
use vers_vecs::BitVec;

use crate::{
    document::Document, info::NodeType, text_usage::TextBuilder, tree_builder::TreeBuilder,
};

struct Parser<R: Read> {
    reader: JsonStreamReader<R>,
    builder: Builder,
}

pub(crate) struct Builder {
    pub(crate) tree_builder: TreeBuilder,
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
}

enum ParseError {
    Reader(ReaderError),
    NumberParseError(ParseFloatError),
}

impl From<ReaderError> for ParseError {
    fn from(err: ReaderError) -> Self {
        ParseError::Reader(err)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        ParseError::NumberParseError(err)
    }
}

impl<R: Read> Parser<R> {
    fn new(json: R) -> Self {
        Self {
            reader: JsonStreamReader::new(json),
            builder: Builder::new(),
        }
    }

    fn parse(mut self) -> Result<Document, ParseError> {
        while self.reader.has_next()? {
            self.parse_item()?;
        }
        let structure = self.builder.tree_builder.build();
        let text_usage = self.builder.text_builder.build();
        Ok(Document::new(
            structure,
            text_usage,
            self.builder.numbers,
            self.builder.booleans,
        ))
    }

    fn parse_item(&mut self) -> Result<(), ParseError> {
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
