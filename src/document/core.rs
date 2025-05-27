use vers_vecs::BitVec;

use crate::{structure::Structure, text_usage::TextUsage};

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
    text_usage: TextUsage,
    numbers: Vec<f64>,
    booleans: BitVec,
}

impl Document {
    pub(crate) fn new(
        structure: Structure,
        text_usage: TextUsage,
        numbers: Vec<f64>,
        booleans: BitVec,
    ) -> Self {
        Self {
            structure,
            text_usage,
            numbers,
            booleans,
        }
    }
}
