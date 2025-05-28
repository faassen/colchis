use struson::writer::{JsonStreamWriter, JsonWriter};

use crate::{info::NodeType, usage::UsageIndex};

use super::{Document, Node, Value};

#[derive(Debug, Clone)]
pub struct ObjectValue<'a, U: UsageIndex> {
    document: &'a Document<U>,
    node: Node,
}

impl<U: UsageIndex> PartialEq for ObjectValue<'_, U> {
    fn eq(&self, other: &Self) -> bool {
        // document reference equality
        self.node == other.node
            && self.document as *const Document<U> == other.document as *const Document<U>
    }
}

impl<'a, U: UsageIndex> IntoIterator for ObjectValue<'a, U> {
    type Item = (&'a str, Value<'a, U>);
    type IntoIter = FieldEntryIterator<'a, U>;

    fn into_iter(self) -> FieldEntryIterator<'a, U> {
        self.iter()
    }
}

impl<'a, U: UsageIndex> ObjectValue<'a, U> {
    pub(crate) fn new(document: &'a Document<U>, node: Node) -> Self {
        Self { document, node }
    }

    pub fn get(&self, key: &str) -> Option<Value<'a, U>> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    pub fn keys(&self) -> FieldKeyIterator<'a, U> {
        FieldKeyIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }

    pub fn values(&self) -> FieldValueIterator<'a, U> {
        FieldValueIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }

    pub fn iter(&self) -> FieldEntryIterator<'a, U> {
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

pub struct FieldKeyIterator<'a, U: UsageIndex> {
    document: &'a Document<U>,
    node: Option<Node>,
}

impl<'a, U: UsageIndex> Iterator for FieldKeyIterator<'a, U> {
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

pub struct FieldValueIterator<'a, U: UsageIndex> {
    document: &'a Document<U>,
    node: Option<Node>,
}

impl<'a, U: UsageIndex> Iterator for FieldValueIterator<'a, U> {
    type Item = Value<'a, U>;

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

pub struct FieldEntryIterator<'a, U: UsageIndex> {
    document: &'a Document<U>,
    node: Option<Node>,
}

impl<'a, U: UsageIndex> Iterator for FieldEntryIterator<'a, U> {
    type Item = (&'a str, Value<'a, U>);

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
