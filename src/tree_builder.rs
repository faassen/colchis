use vers_vecs::BitVec;

use crate::{
    info::{NodeInfo, NodeType},
    lookup::NodeLookup,
};

pub(crate) struct TreeBuilder {
    pub(crate) node_lookup: NodeLookup,
    pub(crate) parentheses: BitVec,
    pub(crate) text_opening_parens: BitVec,
    pub(crate) usage: Vec<u64>,
}

impl TreeBuilder {
    pub(crate) fn new() -> Self {
        Self {
            node_lookup: NodeLookup::new(),
            parentheses: BitVec::new(),
            text_opening_parens: BitVec::new(),
            usage: Vec::new(),
        }
    }

    pub(crate) fn open(&mut self, node_type: NodeType) {
        self.parentheses.append(true);

        match node_type {
            NodeType::String => {
                self.text_opening_parens.append(true);
            }
            _ => {
                self.text_opening_parens.append(false);
            }
        }
        let node_info = NodeInfo::open(node_type);
        let node_info_id = self.node_lookup.register(node_info);
        self.usage.push(node_info_id.id());
    }

    pub(crate) fn close(&mut self, node_type: NodeType) {
        self.parentheses.append(false);
        self.text_opening_parens.append(false);
        let node_info = NodeInfo::close(node_type);
        let node_info_id = self.node_lookup.register(node_info);
        self.usage.push(node_info_id.id());
    }
}
