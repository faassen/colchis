use std::io::Read;

use struson::reader::{JsonReader, JsonStreamReader, ReaderError, ValueType};

use crate::{
    document::Document, info::NodeType, text_usage::TextBuilder, tree_builder::TreeBuilder,
};

struct Parser<R: Read> {
    reader: JsonStreamReader<R>,
    tree_builder: TreeBuilder,
    text_builder: TextBuilder,
}

impl<R: Read> Parser<R> {
    fn new(json: R) -> Self {
        Self {
            reader: JsonStreamReader::new(json),
            tree_builder: TreeBuilder::new(),
            text_builder: TextBuilder::new(),
        }
    }
    fn parse(&mut self) -> Result<Document, ReaderError> {
        while self.reader.has_next()? {
            self.parse_item()?;
        }
        todo!()
    }

    fn parse_item(&mut self) -> Result<(), ReaderError> {
        match self.reader.peek()? {
            ValueType::Array => {
                self.reader.begin_array()?;
                self.tree_builder.open(NodeType::Array);
                while self.reader.has_next()? {
                    self.parse_item()?;
                }
                self.reader.end_array()?;
                self.tree_builder.close(NodeType::Array);
            }
            ValueType::Object => {
                self.reader.begin_object()?;
                self.tree_builder.open(NodeType::Object);
                while self.reader.has_next()? {
                    let key = self.reader.next_name_owned()?;
                    let node_type = NodeType::Field(key);
                    // TODO: we could do away with the clone if we used a cow perhaps
                    self.tree_builder.open(node_type.clone());
                    self.parse_item()?;
                    self.tree_builder.close(node_type);
                }
                self.reader.end_object()?;
                self.tree_builder.close(NodeType::Object);
            }
            ValueType::String => {
                let str = self.reader.next_str()?;
                self.tree_builder.open(NodeType::String);
                self.text_builder.string_node(str);
                self.tree_builder.close(NodeType::String);
            }
            ValueType::Number => {}
            ValueType::Boolean => {}
            ValueType::Null => {}
        }
        Ok(())
    }
}
