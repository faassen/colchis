use crate::{
    info::{NodeInfo, NodeInfoId, NodeType},
    lookup::NodeLookup,
};

// TODO: these traits should be sealed somehow

pub trait UsageBuilder {
    fn new() -> Self;

    fn heap_size(&self) -> usize;

    fn node_lookup_mut(&mut self) -> &mut NodeLookup;

    fn open(&mut self, node_type: NodeType) {
        let node_info = NodeInfo::open(node_type);
        let node_info_id = self.node_lookup_mut().register(node_info);
        self.append(node_info_id);
    }

    fn close(&mut self, node_type: NodeType) {
        let node_info = NodeInfo::close(node_type);
        let node_info_id = self.node_lookup_mut().register(node_info);
        self.append(node_info_id);
    }

    fn append(&mut self, node_info_id: NodeInfoId);
}

pub trait UsageIndex {
    fn heap_size(&self) -> usize;

    fn node_lookup(&self) -> &NodeLookup;
    /// The node info id at a position i in the structure.
    fn node_info_id(&self, i: usize) -> Option<NodeInfoId>;

    fn rank(&self, i: usize, node_info_id: NodeInfoId) -> Option<usize>;
    fn select(&self, i: usize, node_info_id: NodeInfoId) -> Option<usize>;

    fn text_id(&self, i: usize) -> Option<usize>;
    fn number_id(&self, i: usize) -> Option<usize>;
    fn boolean_id(&self, i: usize) -> Option<usize>;
}
