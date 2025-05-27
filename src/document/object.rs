use super::{Document, Node};

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

impl<'a> ObjectValue<'a> {
    pub(crate) fn new(document: &'a Document, node: Node) -> Self {
        Self { document, node }
    }
}
