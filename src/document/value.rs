use crate::{info::NodeType, text_usage::TextId};

use super::{Document, Node, ObjectValue, array::ArrayValue};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Object(ObjectValue<'a>),
    Array(ArrayValue<'a>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

impl Document {
    pub fn value(&self, node: Node) -> Value<'_> {
        match self.node_type(node) {
            NodeType::Object => {
                let object_value = self.object_value(node);
                Value::Object(object_value)
            }
            NodeType::Array => {
                let array_value = self.array_value(node);
                Value::Array(array_value)
            }
            NodeType::String => {
                let s = self.string_value(node);
                Value::String(s)
            }
            NodeType::Number => Value::Number(self.number_value(node)),
            NodeType::Boolean => Value::Boolean(self.boolean_value(node)),
            NodeType::Null => Value::Null,
            NodeType::Field(_s) => {
                unreachable!()
            }
        }
    }
    pub fn root_value(&self) -> Value<'_> {
        let root = self.root();
        self.value(root)
    }

    fn string_value(&self, node: Node) -> &str {
        let text_id = self.structure.text_id(node.get()).unwrap();
        let text_id = TextId::new(text_id);
        self.text_usage.text_value(text_id)
    }

    fn number_value(&self, node: Node) -> f64 {
        let number_id = self.structure.number_id(node.get()).unwrap();
        self.numbers[number_id]
    }

    fn boolean_value(&self, node: Node) -> bool {
        let boolean_id = self.structure.boolean_id(node.get()).unwrap();
        self.booleans.is_bit_set_unchecked(boolean_id)
    }

    fn array_value(&self, node: Node) -> ArrayValue<'_> {
        ArrayValue::new(self, node)
    }

    fn object_value(&self, node: Node) -> ObjectValue<'_> {
        ObjectValue::new(self, node)
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

    #[test]
    fn test_string_value() {
        let doc = Document::parse(r#""hello""#.as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::String("hello"));
    }

    #[test]
    fn test_array() {
        let doc = Document::parse(r#"["a", "b", "c"]"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Array(array_value) = v {
            let mut iter = array_value.into_iter();
            assert_eq!(iter.next(), Some(Value::String("a")));
            assert_eq!(iter.next(), Some(Value::String("b")));
            assert_eq!(iter.next(), Some(Value::String("c")));
            assert_eq!(iter.next(), None);
        } else {
            panic!("Expected an array value");
        }
    }

    #[test]
    fn test_nested_array() {
        let doc = Document::parse(r#"[1, [2, 3], 4]"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Array(array_value) = v {
            let mut iter = array_value.into_iter();
            assert_eq!(iter.next(), Some(Value::Number(1.0)));

            if let Some(Value::Array(inner_array)) = iter.next() {
                let mut inner_iter = inner_array.into_iter();
                assert_eq!(inner_iter.next(), Some(Value::Number(2.0)));
                assert_eq!(inner_iter.next(), Some(Value::Number(3.0)));
                assert_eq!(inner_iter.next(), None);
            } else {
                panic!("Expected an inner array value");
            }

            assert_eq!(iter.next(), Some(Value::Number(4.0)));
            assert_eq!(iter.next(), None);
        } else {
            panic!("Expected an array value");
        }
    }

    #[test]
    fn test_object() {
        let doc = Document::parse(r#"{"key1": "value1", "key2": 42}"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Object(object_value) = v {
            assert_eq!(object_value.get("key1"), Some(Value::String("value1")));
            assert_eq!(object_value.get("key2"), Some(Value::Number(42.0)));
        } else {
            panic!("Expected an object value");
        }
    }
}
