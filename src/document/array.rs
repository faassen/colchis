use std::io::Write;

use struson::writer::{JsonStreamWriter, JsonWriter};

use crate::usage::UsageIndex;

use super::{Document, Node, value::Value};

#[derive(Debug, Clone)]
pub struct ArrayValue<'a, U: UsageIndex> {
    document: &'a Document<U>,
    node: Node,
}

impl<U: UsageIndex> PartialEq for ArrayValue<'_, U> {
    fn eq(&self, other: &Self) -> bool {
        // document reference equality
        self.node == other.node
            && self.document as *const Document<U> == other.document as *const Document<U>
    }
}

impl<'a, U: UsageIndex> IntoIterator for ArrayValue<'a, U> {
    type Item = Value<'a, U>;
    type IntoIter = ArrayIterator<'a, U>;

    fn into_iter(self) -> ArrayIterator<'a, U> {
        self.iter()
    }
}

impl<'a, U: UsageIndex> ArrayValue<'a, U> {
    pub(crate) fn new(document: &'a Document<U>, node: Node) -> Self {
        Self { document, node }
    }

    fn iter(&self) -> ArrayIterator<'a, U> {
        ArrayIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }

    pub fn serialize<W: Write>(&self, writer: &mut JsonStreamWriter<W>) -> std::io::Result<()> {
        writer.begin_array()?;
        for value in self.iter() {
            value.serialize(writer)?;
        }
        writer.end_array()
    }
}

pub struct ArrayIterator<'a, U: UsageIndex> {
    document: &'a Document<U>,
    node: Option<Node>,
}

impl<'a, U: UsageIndex> Iterator for ArrayIterator<'a, U> {
    type Item = Value<'a, U>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            self.node = self.document.primitive_next_sibling(node);
            Some(self.document.value(node))
        } else {
            None
        }
    }
}
