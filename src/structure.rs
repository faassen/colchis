use vers_vecs::{BpTree, RsVec};

use crate::{builder::Builder, lookup::NodeLookup};

pub(crate) struct Structure {
    node_lookup: NodeLookup,
    text_opening_parens: RsVec,
    tree: BpTree,
}

impl Structure {
    pub(crate) fn new(builder: Builder) -> Self {
        let node_lookup = builder.node_lookup;
        let text_opening_parens = RsVec::from_bit_vec(builder.text_opening_parens);
        let tree = BpTree::from_bit_vector(builder.parentheses);

        Self {
            node_lookup,
            text_opening_parens,
            tree,
        }
    }
}
