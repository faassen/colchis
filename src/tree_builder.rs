use vers_vecs::BitVec;

use crate::{
    info::{NodeInfo, NodeInfoId, NodeType},
    lookup::NodeLookup,
    structure::Structure,
};

pub(crate) struct TreeBuilder {
    pub(crate) node_lookup: NodeLookup,
    pub(crate) parentheses: BitVec,
    pub(crate) usage: Vec<Vec<u64>>,
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
        self.node_lookup.heap_size() + self.parentheses.heap_size() + self.usage_heap_size()
    }

    pub(crate) fn usage_heap_size(&self) -> usize {
        // a sum of u64 vectors
        self.usage
            .iter()
            .map(|v| v.len() * std::mem::size_of::<u64>())
            .sum::<usize>()
    }

    pub(crate) fn display_heap_sizes(&self) {
        let node_lookup_heap_size = self.node_lookup.heap_size();
        let parentheses_heap_size = self.parentheses.heap_size();
        let usage_heap_size = self.usage_heap_size();

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
        let node_info = NodeInfo::open(node_type);
        let node_info_id = self.node_lookup.register(node_info);
        self.append_usage(node_info_id);
        self.parentheses.append(true);
    }

    pub(crate) fn close(&mut self, node_type: NodeType) {
        let node_info = NodeInfo::close(node_type);
        let node_info_id = self.node_lookup.register(node_info);
        self.append_usage(node_info_id);
        self.parentheses.append(false);
    }

    fn append_usage(&mut self, node_info_id: NodeInfoId) {
        // get the positions for this node_info_id; make it an empty vec if it doesn't exist yet
        let i = node_info_id.id() as usize;
        if self.usage.len() <= i {
            self.usage.resize(i + 1, Vec::new());
        }
        let positions = self.usage.get_mut(i).expect("Entry should be present");
        // note that we should push parentheses after we push usage
        positions.push(self.parentheses.len() as u64)
    }

    pub(crate) fn build(self) -> Structure {
        Structure::new(self)
    }
}
