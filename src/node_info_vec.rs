use vers_vecs::SparseRSVec;

use crate::info::NodeInfoId;

pub struct NodeInfoVec {
    sparse_rs_vecs: Vec<SparseRSVec>,
    len: usize,
}

impl NodeInfoVec {
    pub(crate) fn new(tags_usage: &[u64], amount: usize) -> Self {
        let mut all_positions: Vec<Vec<u64>> = vec![vec![]; amount];
        for (i, entry) in tags_usage.iter().enumerate() {
            let positions = all_positions
                .get_mut(*entry as usize)
                .expect("Entry should be present");
            positions.push(i as u64);
        }
        let sparse_rs_vecs = all_positions
            .into_iter()
            .map(|positions| SparseRSVec::new(&positions, tags_usage.len() as u64))
            .collect();
        Self {
            sparse_rs_vecs,
            len: tags_usage.len(),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank() {
        let v = NodeInfoVec::new(&[0, 1, 1, 3, 2, 3], 4);
        assert_eq!(v.rank_node_info_id(0, NodeInfoId::new(0)), Some(0));
        assert_eq!(v.rank_node_info_id(1, NodeInfoId::new(0)), Some(1));
        assert_eq!(v.rank_node_info_id(2, NodeInfoId::new(1)), Some(1));
        assert_eq!(v.rank_node_info_id(3, NodeInfoId::new(1)), Some(2));
        assert_eq!(v.rank_node_info_id(6, NodeInfoId::new(3)), Some(2));
        assert_eq!(v.rank_node_info_id(10, NodeInfoId::new(3)), None);
    }

    #[test]
    fn test_sa_select() {
        let v = NodeInfoVec::new(&[0, 1, 1, 3, 2, 3], 4);
        assert_eq!(v.select_node_info_id(0, NodeInfoId::new(0)), Some(0));
        assert_eq!(v.select_node_info_id(0, NodeInfoId::new(1)), Some(1));
        assert_eq!(v.select_node_info_id(1, NodeInfoId::new(1)), Some(2));
        assert_eq!(v.select_node_info_id(0, NodeInfoId::new(3)), Some(3));
        assert_eq!(v.select_node_info_id(1, NodeInfoId::new(3)), Some(5));
        assert_eq!(v.select_node_info_id(2, NodeInfoId::new(3)), None);
    }
}
