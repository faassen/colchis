#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeInfoId(u64);

impl NodeInfoId {
    pub fn new(id: u64) -> Self {
        NodeInfoId(id)
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

pub(crate) const OBJECT_OPEN_ID: NodeInfoId = NodeInfoId(0);
pub(crate) const OBJECT_CLOSE_ID: NodeInfoId = NodeInfoId(1);
pub(crate) const ARRAY_OPEN_ID: NodeInfoId = NodeInfoId(2);
pub(crate) const ARRAY_CLOSE_ID: NodeInfoId = NodeInfoId(3);
pub(crate) const STRING_OPEN_ID: NodeInfoId = NodeInfoId(4);
pub(crate) const STRING_CLOSE_ID: NodeInfoId = NodeInfoId(5);
pub(crate) const NUMBER_OPEN_ID: NodeInfoId = NodeInfoId(6);
pub(crate) const NUMBER_CLOSE_ID: NodeInfoId = NodeInfoId(7);
pub(crate) const BOOLEAN_OPEN_ID: NodeInfoId = NodeInfoId(8);
pub(crate) const BOOLEAN_CLOSE_ID: NodeInfoId = NodeInfoId(9);
pub(crate) const NULL_OPEN_ID: NodeInfoId = NodeInfoId(10);
pub(crate) const NULL_CLOSE_ID: NodeInfoId = NodeInfoId(11);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
    Field(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeInfo {
    pub node_type: NodeType,
    pub is_open_tag: bool,
}

impl NodeInfo {
    pub(crate) fn open(node_type: NodeType) -> Self {
        NodeInfo {
            node_type,
            is_open_tag: true,
        }
    }

    pub(crate) fn close(node_type: NodeType) -> Self {
        NodeInfo {
            node_type,
            is_open_tag: false,
        }
    }
}
