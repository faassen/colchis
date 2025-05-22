use crate::structure::Structure;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(usize);

impl Node {
    pub(crate) fn new(index: usize) -> Self {
        Node(index)
    }

    pub(crate) fn get(&self) -> usize {
        self.0
    }
}

pub struct Document {
    structure: Structure,
}
