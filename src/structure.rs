use vers_vecs::{BpTree, RsVec};

use crate::{
    info::{NodeInfo, NodeInfoId},
    lookup::NodeLookup,
    node_info_vec::NodeInfoVec,
    tree_builder::TreeBuilder,
};

pub(crate) struct Structure {
    node_lookup: NodeLookup,
    text_opening_parens: RsVec,
    tree: BpTree,
    node_info_vec: NodeInfoVec,
}

impl Structure {
    pub(crate) fn new(builder: TreeBuilder) -> Self {
        let amount = builder.node_lookup.len();
        let node_lookup = builder.node_lookup;
        let text_opening_parens = RsVec::from_bit_vec(builder.text_opening_parens);
        let tree = BpTree::from_bit_vector(builder.parentheses);
        let node_info_vec = NodeInfoVec::new(&builder.usage, amount);
        Self {
            node_lookup,
            text_opening_parens,
            tree,
            node_info_vec,
        }
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
