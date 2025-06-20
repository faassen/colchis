use vers_vecs::BitVec;

use crate::{
    info::{NodeInfoId, NodeType},
    usage::UsageBuilder,
};

pub(crate) struct TreeBuilder<T: UsageBuilder> {
    pub(crate) usage_builder: T,
    pub(crate) parentheses: BitVec,
}

impl<T: UsageBuilder> TreeBuilder<T> {
    pub(crate) fn new() -> Self {
        Self {
            usage_builder: T::new(),
            parentheses: BitVec::new(),
        }
    }

    pub(crate) fn heap_size(&self) -> usize {
        self.usage_builder.heap_size() + self.parentheses.heap_size()
    }

    #[allow(dead_code)]
    pub(crate) fn display_heap_sizes(&self) {
        let usage_builder_heap_size = self.usage_builder.heap_size();
        let parentheses_heap_size = self.parentheses.heap_size();

        println!(
            "Parentheses: {:>15} ({:>6} Mb), Usage: {:>15} ({:>6} Mb)",
            parentheses_heap_size,
            parentheses_heap_size / (1024 * 1024),
            usage_builder_heap_size,
            usage_builder_heap_size / (1024 * 1024)
        );
    }

    pub(crate) fn open(&mut self, node_type: NodeType) {
        self.usage_builder.open(node_type);
        self.parentheses.append(true);
    }

    pub(crate) fn close(&mut self, node_type: NodeType) {
        self.usage_builder.close(node_type);
        self.parentheses.append(false);
    }

    pub(crate) fn open_field(&mut self, name: &str) -> NodeInfoId {
        let close_field_id = self.usage_builder.open_field(name);
        self.parentheses.append(true);
        close_field_id
    }

    pub(crate) fn close_field(&mut self, close_field_id: NodeInfoId) {
        self.usage_builder.close_field(close_field_id);
        self.parentheses.append(false);
    }
}
