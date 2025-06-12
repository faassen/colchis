use std::io::Read;

use vers_vecs::BitVec;

use crate::{
    info::NodeType,
    parser::{JsonParseError, parse},
    structure::Structure,
    text_usage::TextUsage,
    usage::{UsageBuilder, UsageIndex},
};

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

#[derive(Debug)]
pub struct Document<U: UsageIndex> {
    pub(crate) structure: Structure<U>,
    pub(crate) text_usage: TextUsage,
    pub(crate) numbers: Vec<f64>,
    pub(crate) booleans: BitVec,
}

impl<U: UsageIndex> Document<U> {
    pub(crate) fn new(
        structure: Structure<U>,
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

    pub fn heap_size(&self) -> usize {
        self.structure.heap_size()
            + self.text_usage.heap_size()
            + self.numbers.len() * std::mem::size_of::<f64>()
            + self.booleans.heap_size()
    }

    pub fn parse<B: UsageBuilder<Index = U>, R: Read>(
        json: R,
    ) -> Result<Document<B::Index>, JsonParseError> {
        parse::<R, B>(json)
    }

    pub(crate) fn node_type(&self, node: Node) -> &NodeType {
        let node_info = self.structure.node_info(node.get());
        node_info.node_type()
    }
}
