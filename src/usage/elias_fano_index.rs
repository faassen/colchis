use vers_vecs::SparseRSVec;

use super::{roaring_builder::RoaringUsageBuilder, traits::UsageIndex};
use crate::{
    info::{self, NodeInfoId},
    lookup::NodeLookup,
};

#[derive(Debug)]
pub struct EliasFanoUsageIndex {
    sparse_rs_vecs: Vec<SparseRSVec>,
    node_lookup: NodeLookup,
    len: usize,
}

impl EliasFanoUsageIndex {
    pub(crate) fn new(
        sparse_rs_vecs: Vec<SparseRSVec>,
        node_lookup: NodeLookup,
        len: usize,
    ) -> Self {
        Self {
            sparse_rs_vecs,
            node_lookup,
            len,
        }
    }
}

impl UsageIndex for EliasFanoUsageIndex {
    fn heap_size(&self) -> usize {
        self.sparse_rs_vecs.iter().map(|v| v.heap_size()).sum()
    }

    fn node_lookup(&self) -> &NodeLookup {
        &self.node_lookup
    }

    fn node_info_id(&self, i: usize) -> Option<NodeInfoId> {
        // we want to avoid having to store an array of node info ids and the information is already in the sparse rs vecs
        // but is this fast enough?
        for (id, sparse_rs_vec) in self.sparse_rs_vecs.iter().enumerate() {
            if let Some(b) = sparse_rs_vec.is_set(i as u64) {
                if b {
                    return Some(NodeInfoId::new(id as u64));
                }
            }
        }
        None
    }

    fn rank(&self, i: usize, node_info_id: NodeInfoId) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[node_info_id.id() as usize].rank1(i as u64) as usize)
        } else {
            None
        }
    }

    fn select(&self, rank: usize, node_info_id: NodeInfoId) -> Option<usize> {
        let s = self.sparse_rs_vecs[node_info_id.id() as usize].select1(rank) as usize;
        if self.len != s { Some(s) } else { None }
    }

    fn text_id(&self, i: usize) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[info::STRING_OPEN_ID.index()].rank1(i as u64) as usize)
        } else {
            None
        }
    }

    // in sparse bit vec for opening number, we can do a rank check to determine
    // the number id
    fn number_id(&self, i: usize) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[info::NUMBER_OPEN_ID.index()].rank1(i as u64) as usize)
        } else {
            None
        }
    }

    fn boolean_id(&self, i: usize) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[info::BOOLEAN_OPEN_ID.index()].rank1(i as u64) as usize)
        } else {
            None
        }
    }
}
