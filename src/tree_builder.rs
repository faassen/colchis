use vers_vecs::BitVec;

use crate::{
    info::{NodeInfo, NodeType},
    lookup::NodeLookup,
    structure::Structure,
};

pub(crate) struct TreeBuilder {
    pub(crate) node_lookup: NodeLookup,
    pub(crate) parentheses: BitVec,
    pub(crate) usage: Vec<u64>,
}

impl TreeBuilder {
    pub(crate) fn new() -> Self {
        Self {
            node_lookup: NodeLookup::new(),
            parentheses: BitVec::new(),
            usage: Vec::new(),
        }
    }

    pub(crate) fn heap_size(&self) -> usize {
        self.node_lookup.heap_size()
            + self.parentheses.heap_size()
            + self.usage.len() * std::mem::size_of::<u64>()
    }

    pub(crate) fn display_heap_sizes(&self) {
        let node_lookup_heap_size = self.node_lookup.heap_size();
        let parentheses_heap_size = self.parentheses.heap_size();
        let usage_heap_size = self.usage.len() * std::mem::size_of::<u64>();

        println!(
            "Node lookup: {:>15} ({:>6} Mb), Parentheses: {:>15} ({:>6} Mb), Usage: {:>15} ({:>6} Mb)",
            node_lookup_heap_size,
            node_lookup_heap_size / 1_000_000,
            parentheses_heap_size,
            parentheses_heap_size / 1_000_000,
            usage_heap_size,
            usage_heap_size / 1_000_000
        );
    }

    pub(crate) fn open(&mut self, node_type: NodeType) {
        self.parentheses.append(true);
        let node_info = NodeInfo::open(node_type);
        let node_info_id = self.node_lookup.register(node_info);
        self.usage.push(node_info_id.id());
    }

    pub(crate) fn close(&mut self, node_type: NodeType) {
        self.parentheses.append(false);
        let node_info = NodeInfo::close(node_type);
        let node_info_id = self.node_lookup.register(node_info);
        self.usage.push(node_info_id.id());
    }

    pub(crate) fn build(self) -> Structure {
        Structure::new(self)
    }
}
