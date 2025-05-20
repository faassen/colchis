use ahash::HashMap;

use crate::info::{self, NodeInfo, NodeInfoId, NodeType};

pub(crate) struct NodeLookup {
    node_infos: Vec<NodeInfo>,
    node_info_lookup: HashMap<NodeInfo, NodeInfoId>,
}

impl NodeLookup {
    pub fn new() -> Self {
        let mut node_lookup = Self {
            node_infos: Vec::new(),
            node_info_lookup: HashMap::default(),
        };

        // register the hardcoded node ids so we can skip using the
        // hashmap for them during lookup
        let object_node_info_open_id =
            node_lookup.register_lookup(NodeInfo::open(NodeType::Object));
        debug_assert_eq!(object_node_info_open_id.id(), info::OBJECT_OPEN_ID.id());
        let object_node_info_close_id =
            node_lookup.register_lookup(NodeInfo::close(NodeType::Object));
        debug_assert_eq!(object_node_info_close_id.id(), info::OBJECT_CLOSE_ID.id());

        let array_node_info_open_id = node_lookup.register_lookup(NodeInfo::open(NodeType::Array));
        debug_assert_eq!(array_node_info_open_id.id(), info::ARRAY_OPEN_ID.id());
        let array_node_info_close_id =
            node_lookup.register_lookup(NodeInfo::close(NodeType::Array));
        debug_assert_eq!(array_node_info_close_id.id(), info::ARRAY_CLOSE_ID.id());

        let string_node_info_open_id =
            node_lookup.register_lookup(NodeInfo::open(NodeType::String));
        debug_assert_eq!(string_node_info_open_id.id(), info::STRING_OPEN_ID.id());
        let string_node_info_close_id =
            node_lookup.register_lookup(NodeInfo::close(NodeType::String));
        debug_assert_eq!(string_node_info_close_id.id(), info::STRING_CLOSE_ID.id());

        let number_node_info_open_id =
            node_lookup.register_lookup(NodeInfo::open(NodeType::Number));
        debug_assert_eq!(number_node_info_open_id.id(), info::NUMBER_OPEN_ID.id());
        let number_node_info_close_id =
            node_lookup.register_lookup(NodeInfo::close(NodeType::Number));
        debug_assert_eq!(number_node_info_close_id.id(), info::NUMBER_CLOSE_ID.id());

        let boolean_node_info_open_id =
            node_lookup.register_lookup(NodeInfo::open(NodeType::Boolean));
        debug_assert_eq!(boolean_node_info_open_id.id(), info::BOOLEAN_OPEN_ID.id());
        let boolean_node_info_close_id =
            node_lookup.register_lookup(NodeInfo::close(NodeType::Boolean));
        debug_assert_eq!(boolean_node_info_close_id.id(), info::BOOLEAN_CLOSE_ID.id());

        let null_node_info_open_id = node_lookup.register_lookup(NodeInfo::open(NodeType::Null));
        debug_assert_eq!(null_node_info_open_id.id(), info::NULL_OPEN_ID.id());
        let null_node_info_close_id = node_lookup.register_lookup(NodeInfo::close(NodeType::Null));
        debug_assert_eq!(null_node_info_close_id.id(), info::NULL_CLOSE_ID.id());

        node_lookup
    }

    pub fn register(&mut self, node_info: NodeInfo) -> NodeInfoId {
        if let Some(idx) = self.register_fast_path(&node_info) {
            return idx;
        }
        self.register_lookup(node_info)
    }

    fn register_fast_path(&mut self, node_info: &NodeInfo) -> Option<NodeInfoId> {
        Some(match (node_info.is_open_tag, &node_info.node_type) {
            (true, NodeType::Object) => info::OBJECT_OPEN_ID,
            (false, NodeType::Object) => info::OBJECT_CLOSE_ID,
            (true, NodeType::Array) => info::ARRAY_OPEN_ID,
            (false, NodeType::Array) => info::ARRAY_CLOSE_ID,
            (true, NodeType::String) => info::STRING_OPEN_ID,
            (false, NodeType::String) => info::STRING_CLOSE_ID,
            (true, NodeType::Number) => info::NUMBER_OPEN_ID,
            (false, NodeType::Number) => info::NUMBER_CLOSE_ID,
            (true, NodeType::Boolean) => info::BOOLEAN_OPEN_ID,
            (false, NodeType::Boolean) => info::BOOLEAN_CLOSE_ID,
            (true, NodeType::Null) => info::NULL_OPEN_ID,
            (false, NodeType::Null) => info::NULL_CLOSE_ID,
            _ => return None,
        })
    }

    pub(crate) fn register_lookup(&mut self, node_info: NodeInfo) -> NodeInfoId {
        if let Some(&idx) = self.node_info_lookup.get(&node_info) {
            return idx;
        }
        let idx = NodeInfoId::new(self.node_infos.len() as u64);
        self.node_infos.push(node_info.clone());
        self.node_info_lookup.insert(node_info, idx);
        idx
    }

    pub(crate) fn by_node_info(&self, node_info: &NodeInfo) -> Option<NodeInfoId> {
        self.node_info_lookup.get(node_info).copied()
    }

    pub(crate) fn by_node_info_id(&self, node_info_id: NodeInfoId) -> &NodeInfo {
        self.node_infos
            .get(node_info_id.id() as usize)
            .expect("Node info id does not exist in this document")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_predefined_nodes() {
        let mut lookup = NodeLookup::new();

        // Test object nodes
        let object_open = NodeInfo::open(NodeType::Object);
        let object_close = NodeInfo::close(NodeType::Object);

        let object_open_id = lookup.register(object_open.clone());
        let object_close_id = lookup.register(object_close.clone());

        assert_eq!(object_open_id, info::OBJECT_OPEN_ID);
        assert_eq!(object_close_id, info::OBJECT_CLOSE_ID);

        // Test array nodes
        let array_open = NodeInfo::open(NodeType::Array);
        let array_close = NodeInfo::close(NodeType::Array);

        let array_open_id = lookup.register(array_open.clone());
        let array_close_id = lookup.register(array_close.clone());

        assert_eq!(array_open_id, info::ARRAY_OPEN_ID);
        assert_eq!(array_close_id, info::ARRAY_CLOSE_ID);

        // Test string nodes
        let string_open = NodeInfo::open(NodeType::String);
        let string_close = NodeInfo::close(NodeType::String);

        let string_open_id = lookup.register(string_open.clone());
        let string_close_id = lookup.register(string_close.clone());

        assert_eq!(string_open_id, info::STRING_OPEN_ID);
        assert_eq!(string_close_id, info::STRING_CLOSE_ID);

        // Test number nodes
        let number_open = NodeInfo::open(NodeType::Number);
        let number_close = NodeInfo::close(NodeType::Number);

        let number_open_id = lookup.register(number_open.clone());
        let number_close_id = lookup.register(number_close.clone());

        assert_eq!(number_open_id, info::NUMBER_OPEN_ID);
        assert_eq!(number_close_id, info::NUMBER_CLOSE_ID);

        // Test boolean nodes
        let boolean_open = NodeInfo::open(NodeType::Boolean);
        let boolean_close = NodeInfo::close(NodeType::Boolean);

        let boolean_open_id = lookup.register(boolean_open.clone());
        let boolean_close_id = lookup.register(boolean_close.clone());

        assert_eq!(boolean_open_id, info::BOOLEAN_OPEN_ID);
        assert_eq!(boolean_close_id, info::BOOLEAN_CLOSE_ID);

        // Test null nodes
        let null_open = NodeInfo::open(NodeType::Null);
        let null_close = NodeInfo::close(NodeType::Null);

        let null_open_id = lookup.register(null_open.clone());
        let null_close_id = lookup.register(null_close.clone());

        assert_eq!(null_open_id, info::NULL_OPEN_ID);
        assert_eq!(null_close_id, info::NULL_CLOSE_ID);
    }

    #[test]
    fn test_register_field_nodes() {
        let mut lookup = NodeLookup::new();

        // Register field nodes (these should get dynamic IDs)
        let field1 = NodeInfo::open(NodeType::Field("name".to_string()));
        let field2 = NodeInfo::open(NodeType::Field("age".to_string()));

        let field1_id = lookup.register(field1.clone());
        let field2_id = lookup.register(field2.clone());

        // Register same fields again - should get same IDs
        let field1_id_again = lookup.register(field1.clone());
        let field2_id_again = lookup.register(field2.clone());

        assert_eq!(field1_id, field1_id_again);
        assert_eq!(field2_id, field2_id_again);
    }

    #[test]
    fn test_lookup_by_node_info() {
        let mut lookup = NodeLookup::new();

        // Register some nodes
        let object_open = NodeInfo::open(NodeType::Object);
        let field = NodeInfo::open(NodeType::Field("name".to_string()));

        let object_id = lookup.register(object_open.clone());
        let field_id = lookup.register(field.clone());

        // Look them up
        let found_object_id = lookup.by_node_info(&object_open).unwrap();
        let found_field_id = lookup.by_node_info(&field).unwrap();

        assert_eq!(found_object_id, object_id);
        assert_eq!(found_field_id, field_id);

        // Try to look up a non-existent node
        let non_existent = NodeInfo::open(NodeType::Field("does_not_exist".to_string()));
        assert!(lookup.by_node_info(&non_existent).is_none());
    }

    #[test]
    fn test_lookup_by_node_info_id() {
        let mut lookup = NodeLookup::new();

        // Register some nodes
        let object_open = NodeInfo::open(NodeType::Object);
        let field = NodeInfo::open(NodeType::Field("name".to_string()));

        let object_id = lookup.register(object_open.clone());
        let field_id = lookup.register(field.clone());

        // Look them up by ID
        let found_object = lookup.by_node_info_id(object_id);
        let found_field = lookup.by_node_info_id(field_id);

        assert_eq!(found_object, &object_open);
        assert_eq!(found_field, &field);
    }

    #[test]
    fn test_edge_case_empty_fields() {
        let mut lookup = NodeLookup::new();

        // Test empty field name
        let empty_field = NodeInfo::open(NodeType::Field("".to_string()));
        let empty_field_id = lookup.register(empty_field.clone());

        // Register a different empty field (close tag)
        let empty_field_close = NodeInfo::close(NodeType::Field("".to_string()));
        let empty_field_close_id = lookup.register(empty_field_close.clone());

        // Should get a different ID since open/close are different
        assert_ne!(empty_field_id, empty_field_close_id);
    }
}
