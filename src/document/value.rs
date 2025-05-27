use crate::info::NodeType;

use super::{Document, Node};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Object(ObjectValue<'a>),
    Array(ArrayValue<'a>),
    Text(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct ObjectValue<'a> {
    document: &'a Document,
    node: Node,
}

impl PartialEq for ObjectValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        // document reference equality
        self.node == other.node
            && self.document as *const Document == other.document as *const Document
    }
}

#[derive(Debug, Clone)]
pub struct ArrayValue<'a> {
    document: &'a Document,
    node: Node,
}

impl PartialEq for ArrayValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        // document reference equality
        self.node == other.node
            && self.document as *const Document == other.document as *const Document
    }
}

impl Document {
    pub fn value(&self, node: Node) -> Value<'_> {
        match self.node_type(node) {
            NodeType::Object => {
                todo!()
            }
            NodeType::Array => {
                todo!()
            }
            NodeType::String => {
                todo!()
            }
            NodeType::Number => Value::Number(self.number_value(node)),
            NodeType::Boolean => Value::Boolean(self.boolean_value(node)),
            NodeType::Null => Value::Null,
            NodeType::Field(s) => {
                todo!()
            }
        }
    }
    pub fn root_value(&self) -> Value<'_> {
        let root = self.root();
        self.value(root)
    }

    fn number_value(&self, node: Node) -> f64 {
        let number_id = self.structure.number_id(node.get()).unwrap();
        self.numbers[number_id]
    }

    fn boolean_value(&self, node: Node) -> bool {
        let boolean_id = self.structure.boolean_id(node.get()).unwrap();
        self.booleans.is_bit_set_unchecked(boolean_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_value() {
        let doc = Document::parse("42".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Number(42f64));
    }

    #[test]
    fn test_boolean_value_true() {
        let doc = Document::parse("true".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Boolean(true));
    }

    #[test]
    fn test_boolean_value_false() {
        let doc = Document::parse("false".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Boolean(false));
    }

    #[test]
    fn test_null_value() {
        let doc = Document::parse("null".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Null);
    }
}
