use super::{Document, Node, value::Value};

#[derive(Debug, Clone)]
pub struct ArrayValue<'a> {
    document: &'a Document,
    node: Node,
}

impl PartialEq for ArrayValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        // document reference equality
        self.node == other.node
            && self.document as *const Document == other.document as *const Document
    }
}

impl<'a> IntoIterator for ArrayValue<'a> {
    type Item = Value<'a>;
    type IntoIter = ArrayIterator<'a>;

    fn into_iter(self) -> ArrayIterator<'a> {
        self.iter()
    }
}

impl<'a> ArrayValue<'a> {
    pub(crate) fn new(document: &'a Document, node: Node) -> Self {
        Self { document, node }
    }

    fn iter(&self) -> ArrayIterator<'a> {
        ArrayIterator {
            document: self.document,
            node: self.document.primitive_first_child(self.node),
        }
    }
}

pub struct ArrayIterator<'a> {
    document: &'a Document,
    node: Option<Node>,
}

impl<'a> Iterator for ArrayIterator<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.node {
            self.node = self.document.primitive_next_sibling(node);
            Some(self.document.value(node))
        } else {
            None
        }
    }
}
