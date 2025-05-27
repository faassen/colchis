use vers_vecs::Tree;

use super::{Document, Node};

impl Document {
    pub fn root(&self) -> Node {
        Node::new(
            self.structure
                .tree()
                .root()
                .expect("Root node does not exist"),
        )
    }

    pub(crate) fn primitive_parent(&self, node: Node) -> Option<Node> {
        self.structure.tree().parent(node.get()).map(Node::new)
    }

    pub(crate) fn primitive_first_child(&self, node: Node) -> Option<Node> {
        self.structure.tree().first_child(node.get()).map(Node::new)
    }

    pub(crate) fn primitive_next_sibling(&self, node: Node) -> Option<Node> {
        self.structure
            .tree()
            .next_sibling(node.get())
            .map(Node::new)
    }
}
