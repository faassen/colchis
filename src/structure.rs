use vers_vecs::BpTree;

use crate::{
    info::{NodeInfo, NodeInfoId},
    lookup::NodeLookup,
    node_info_vec::NodeInfoVec,
    tree_builder::TreeBuilder,
};

#[derive(Debug)]
pub(crate) struct Structure {
    node_lookup: NodeLookup,
    tree: BpTree,
    node_info_vec: NodeInfoVec,
}

impl Structure {
    pub(crate) fn new(tree_builder: TreeBuilder) -> Self {
        let amount = tree_builder.node_lookup.len();
        let node_lookup = tree_builder.node_lookup;
        let tree = BpTree::from_bit_vector(tree_builder.parentheses);
        let node_info_vec = NodeInfoVec::new(tree_builder.usage, amount);

        Self {
            node_lookup,
            tree,
            node_info_vec,
        }
    }

    pub(crate) fn heap_size(&self) -> usize {
        self.tree.heap_size() + self.node_lookup.heap_size() + self.node_info_vec.heap_size()
    }

    pub(crate) fn lookup_node_info(&self, node_info_id: NodeInfoId) -> &NodeInfo {
        self.node_lookup.by_node_info_id(node_info_id)
    }

    pub(crate) fn node_info(&self, i: usize) -> &NodeInfo {
        let id = self.node_info_id(i);
        self.lookup_node_info(id)
    }

    pub(crate) fn node_info_id(&self, i: usize) -> NodeInfoId {
        self.node_info_vec
            .node_info_id(i)
            .expect("Node information to exist")
    }

    pub(crate) fn tree(&self) -> &BpTree {
        &self.tree
    }

    pub(crate) fn text_id(&self, i: usize) -> Option<usize> {
        self.node_info_vec.text_id(i)
    }

    pub(crate) fn number_id(&self, i: usize) -> Option<usize> {
        self.node_info_vec.number_id(i)
    }

    pub(crate) fn boolean_id(&self, i: usize) -> Option<usize> {
        self.node_info_vec.boolean_id(i)
    }
}

#[cfg(test)]
mod tests {
    use crate::info::{NodeInfo, NodeType};

    use super::*;

    #[test]
    fn test_structure() {
        let mut builder = TreeBuilder::new();

        // ["a", "b", "c"]
        builder.open(NodeType::Array);
        builder.open(NodeType::String);
        builder.close(NodeType::String);
        builder.open(NodeType::String);
        builder.close(NodeType::String);
        builder.open(NodeType::String);
        builder.close(NodeType::String);
        builder.close(NodeType::String);

        let structure = Structure::new(builder);

        assert_eq!(structure.node_info(0), &NodeInfo::open(NodeType::Array));
        assert_eq!(structure.node_info(1), &NodeInfo::open(NodeType::String));
    }
}
