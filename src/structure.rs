use vers_vecs::BpTree;

use crate::{
    info::{NodeInfo, NodeInfoId},
    tree_builder::TreeBuilder,
    usage::{UsageBuilder, UsageIndex},
};

#[derive(Debug)]
pub(crate) struct Structure<T: UsageIndex> {
    usage_index: T,
    tree: BpTree,
}

impl<U: UsageIndex> Structure<U> {
    pub(crate) fn new<B: UsageBuilder<Index = U>>(tree_builder: TreeBuilder<B>) -> Self {
        let tree = BpTree::from_bit_vector(tree_builder.parentheses);
        let usage_index = tree_builder.usage_builder.build();

        Self { usage_index, tree }
    }

    pub(crate) fn heap_size(&self) -> usize {
        self.tree.heap_size() + self.usage_index.heap_size()
    }

    pub(crate) fn lookup_node_info(&self, node_info_id: NodeInfoId) -> &NodeInfo {
        self.usage_index.node_lookup().by_node_info_id(node_info_id)
    }

    pub(crate) fn node_info(&self, i: usize) -> &NodeInfo {
        let id = self.node_info_id(i);
        self.lookup_node_info(id)
    }

    pub(crate) fn node_info_id(&self, i: usize) -> NodeInfoId {
        self.usage_index
            .node_info_id(i)
            .expect("Node information does not exist")
    }

    pub(crate) fn tree(&self) -> &BpTree {
        &self.tree
    }

    pub(crate) fn text_id(&self, i: usize) -> Option<usize> {
        self.usage_index.text_id(i)
    }

    pub(crate) fn number_id(&self, i: usize) -> Option<usize> {
        self.usage_index.number_id(i)
    }

    pub(crate) fn boolean_id(&self, i: usize) -> Option<usize> {
        self.usage_index.boolean_id(i)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        info::{NodeInfo, NodeType},
        usage::{EliasFanoUsageIndex, RoaringUsageBuilder},
    };

    use super::*;

    #[test]
    fn test_structure() {
        let mut builder = TreeBuilder::<RoaringUsageBuilder>::new();

        // ["a", "b", "c"]
        builder.open(NodeType::Array);
        builder.open(NodeType::String);
        builder.close(NodeType::String);
        builder.open(NodeType::String);
        builder.close(NodeType::String);
        builder.open(NodeType::String);
        builder.close(NodeType::String);
        builder.close(NodeType::String);

        let structure = Structure::<EliasFanoUsageIndex>::new(builder);

        assert_eq!(structure.node_info(0), &NodeInfo::open(NodeType::Array));
        assert_eq!(structure.node_info(1), &NodeInfo::open(NodeType::String));
    }
}
