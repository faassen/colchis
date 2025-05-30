use roaring::RoaringBitmap;
use vers_vecs::SparseRSVec;

use crate::{info::NodeInfoId, lookup::NodeLookup};

use super::{EliasFanoUsageIndex, traits::UsageBuilder};

pub struct RoaringUsageBuilder {
    pub(crate) usage: Vec<RoaringBitmap>,
    pub(crate) node_lookup: NodeLookup,
    pub(crate) len: usize,
}

impl RoaringUsageBuilder {
    pub(crate) fn usage_heap_size(&self) -> usize {
        self.usage
            .iter()
            .map(|v| {
                let stats = v.statistics();
                (stats.n_bytes_array_containers
                    + stats.n_bytes_run_containers
                    + stats.n_bytes_bitset_containers) as usize
            })
            .sum::<usize>()
    }
}

impl UsageBuilder for RoaringUsageBuilder {
    type Index = EliasFanoUsageIndex;

    fn new() -> Self {
        Self {
            usage: Vec::new(),
            node_lookup: NodeLookup::new(),
            len: 0,
        }
    }

    fn heap_size(&self) -> usize {
        self.node_lookup.heap_size() + self.usage_heap_size()
    }

    fn node_lookup_mut(&mut self) -> &mut NodeLookup {
        &mut self.node_lookup
    }

    fn append(&mut self, node_info_id: NodeInfoId) {
        // get the positions for this node_info_id; make it an empty vec if it doesn't exist yet
        let i = node_info_id.id() as usize;
        if self.usage.len() <= i {
            self.usage.resize(i + 1, RoaringBitmap::new());
        }
        let positions = self.usage.get_mut(i).expect("Entry should be present");
        // TODO: fail if we go over u32
        positions.push(self.len as u32);
        self.len += 1;
    }

    fn build(self) -> Self::Index {
        // TODO: drain the usage so we can throw away memory early?
        let sparse_rs_vecs = self
            .usage
            .into_iter()
            .map(|bm| {
                let positions = bm.into_iter().map(|i| i as u64).collect::<Vec<u64>>();
                SparseRSVec::new(&positions, self.len as u64)
            })
            .collect();
        Self::Index::new(sparse_rs_vecs, self.node_lookup, self.len)
    }
}
