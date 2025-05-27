use vers_vecs::SparseRSVec;

use crate::info::{self, NodeInfoId};

#[derive(Debug)]
pub struct NodeInfoVec {
    sparse_rs_vecs: Vec<SparseRSVec>,
    len: usize,
}

impl NodeInfoVec {
    pub(crate) fn new(usage: Vec<Vec<u64>>, amount: usize) -> Self {
        let sparse_rs_vecs = usage
            .into_iter()
            .map(|positions| SparseRSVec::new(&positions, amount as u64))
            .collect();
        Self {
            sparse_rs_vecs,
            len: amount,
        }
    }

    pub(crate) fn heap_size(&self) -> usize {
        self.sparse_rs_vecs.iter().map(|v| v.heap_size()).sum()
    }

    // We'd like to minimize the use of this operation in loops
    // but we can't, as node_type depends on it and it's going to be used
    // throughout in the tree API.
    //
    // Maybe this is fast enough if there aren't a lot of keys, after all
    // each individual is_set check is basically constant time.
    //
    // The simplest would be to store a vector of
    // the node ids, but this means an extra integer (possibly a short one) per
    // node. is there something smarter we could do?
    // Unrolled checking of the bitvecs which have a constant might help
    // a bit but doesn't avoid the internal work that spare_rs_vec does.
    //
    // We could store some bits per node id to cut the search time down to
    // only a section of this
    pub(crate) fn node_info_id(&self, i: usize) -> Option<NodeInfoId> {
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

    pub(crate) fn rank_node_info_id(&self, i: usize, node_info_id: NodeInfoId) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[node_info_id.id() as usize].rank1(i as u64) as usize)
        } else {
            None
        }
    }

    pub(crate) fn select_node_info_id(
        &self,
        rank: usize,
        node_info_id: NodeInfoId,
    ) -> Option<usize> {
        let s = self.sparse_rs_vecs[node_info_id.id() as usize].select1(rank) as usize;
        if self.len != s { Some(s) } else { None }
    }

    pub(crate) fn text_id(&self, i: usize) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[info::STRING_OPEN_ID.index()].rank1(i as u64) as usize)
        } else {
            None
        }
    }

    // in sparse bit vec for opening number, we can do a rank check to determine
    // the number id
    pub(crate) fn number_id(&self, i: usize) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[info::NUMBER_OPEN_ID.index()].rank1(i as u64) as usize)
        } else {
            None
        }
    }

    pub(crate) fn boolean_id(&self, i: usize) -> Option<usize> {
        if i <= self.len {
            Some(self.sparse_rs_vecs[info::BOOLEAN_OPEN_ID.index()].rank1(i as u64) as usize)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_info() {
        // 0, 1, 2, 3
        let v = NodeInfoVec::new(vec![vec![0], vec![1], vec![2], vec![3]], 4);
        assert_eq!(v.node_info_id(0), Some(NodeInfoId::new(0)));
        assert_eq!(v.node_info_id(1), Some(NodeInfoId::new(1)));
        assert_eq!(v.node_info_id(2), Some(NodeInfoId::new(2)));
        assert_eq!(v.node_info_id(10), None);
    }

    #[test]
    fn test_rank() {
        // 0, 1, 1, 3, 2, 3
        let v = vec![vec![0], vec![1, 2], vec![4], vec![3, 5]];
        let v = NodeInfoVec::new(v, 6);
        assert_eq!(v.rank_node_info_id(0, NodeInfoId::new(0)), Some(0));
        assert_eq!(v.rank_node_info_id(1, NodeInfoId::new(0)), Some(1));
        assert_eq!(v.rank_node_info_id(2, NodeInfoId::new(1)), Some(1));
        assert_eq!(v.rank_node_info_id(3, NodeInfoId::new(1)), Some(2));
        assert_eq!(v.rank_node_info_id(6, NodeInfoId::new(3)), Some(2));
        assert_eq!(v.rank_node_info_id(10, NodeInfoId::new(3)), None);
    }

    #[test]
    fn test_sa_select() {
        // 0, 1, 1, 3, 2, 3
        let v = vec![vec![0], vec![1, 2], vec![4], vec![3, 5]];
        let v = NodeInfoVec::new(v, 6);
        assert_eq!(v.select_node_info_id(0, NodeInfoId::new(0)), Some(0));
        assert_eq!(v.select_node_info_id(0, NodeInfoId::new(1)), Some(1));
        assert_eq!(v.select_node_info_id(1, NodeInfoId::new(1)), Some(2));
        assert_eq!(v.select_node_info_id(0, NodeInfoId::new(3)), Some(3));
        assert_eq!(v.select_node_info_id(1, NodeInfoId::new(3)), Some(5));
        assert_eq!(v.select_node_info_id(2, NodeInfoId::new(3)), None);
    }
}
