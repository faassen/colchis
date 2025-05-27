use struson::writer::{JsonStreamWriter, JsonWriter};

use crate::info::NodeType;

use super::{Document, Node, Value};

#[derive(Debug, Clone)]
pub struct ObjectValue<'a> {
    document: &'a Document,
    node: Node,
}

impl PartialEq for ObjectValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        // document reference equality
        self.node == other.node
            && self.document as *const Document == other.document as *const Document
    }
}

impl<'a> IntoIterator for ObjectValue<'a> {
    type Item = (&'a str, Value<'a>);
    type IntoIter = FieldEntryIterator<'a>;

    fn into_iter(self) -> FieldEntryIterator<'a> {
        self.iter()
    }
}

impl<'a> ObjectValue<'a> {
    pub(crate) fn new(document: &'a Document, node: Node) -> Self {
        Self { document, node }
    }

    pub fn get(&self, key: &str) -> Option<Value<'a>> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    pub fn keys(&self) -> FieldKeyIterator<'a> {
        FieldKeyIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }

    pub fn values(&self) -> FieldValueIterator<'a> {
        FieldValueIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }

    pub fn iter(&self) -> FieldEntryIterator<'a> {
        FieldEntryIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }

    pub fn serialize<W: std::io::Write>(
        &self,
        writer: &mut JsonStreamWriter<W>,
    ) -> std::io::Result<()> {
        writer.begin_object()?;
        for (key, value) in self.iter() {
            writer.name(key)?;
            value.serialize(writer)?;
        }
        writer.end_object()
    }
}

pub struct FieldKeyIterator<'a> {
    document: &'a Document,
    node: Option<Node>,
}

impl<'a> Iterator for FieldKeyIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            self.node = self.document.primitive_next_sibling(node);
            let node_type = self.document.node_type(node);
            if let NodeType::Field(key) = node_type {
                Some(key)
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
}

pub struct FieldValueIterator<'a> {
    document: &'a Document,
    node: Option<Node>,
}

impl<'a> Iterator for FieldValueIterator<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            // we go to the next field
            self.node = self.document.primitive_next_sibling(node);
            // now we get the value of the first child of the field node
            let value_node = self.document.primitive_first_child(node).unwrap();
            Some(self.document.value(value_node))
        } else {
            None
        }
    }
}

pub struct FieldEntryIterator<'a> {
    document: &'a Document,
    node: Option<Node>,
}

impl<'a> Iterator for FieldEntryIterator<'a> {
    type Item = (&'a str, Value<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            // we go to the next field
            self.node = self.document.primitive_next_sibling(node);
            // now we get the key and value of the field node
            let node_type = self.document.node_type(node);
            if let NodeType::Field(key) = node_type {
                let value_node = self.document.primitive_first_child(node).unwrap();
                Some((key, self.document.value(value_node)))
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
}
