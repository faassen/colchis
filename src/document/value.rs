use std::io::Write;
use std::sync::Arc;

use struson::writer::{JsonStreamWriter, JsonWriter};

use crate::{info::NodeType, text::TextId, usage::UsageIndex};

use super::{Document, Node, ObjectValue, array::ArrayValue};

#[derive(Debug, Clone)]
pub enum Value<'a, U: UsageIndex> {
    Object(ObjectValue<'a, U>),
    Array(ArrayValue<'a, U>),
    String(Arc<str>),
    Number(f64),
    Boolean(bool),
    Null,
}

impl<U: UsageIndex> PartialEq for Value<'_, U> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl<U: UsageIndex> Value<'_, U> {
    pub fn serialize<W: Write>(&self, writer: &mut JsonStreamWriter<W>) -> std::io::Result<()> {
        match self {
            Value::Object(object) => object.serialize(writer),
            Value::Array(array) => array.serialize(writer),
            Value::String(s) => writer.string_value(&s),
            Value::Number(n) => match writer.fp_number_value(*n) {
                Ok(_) => Ok(()),
                Err(e) => match e {
                    struson::writer::JsonNumberError::IoError(e) => Err(e),
                    _ => {
                        unreachable!();
                    }
                },
            },
            Value::Boolean(b) => writer.bool_value(*b),
            Value::Null => writer.null_value(),
        }
    }
}

impl<U: UsageIndex> Document<U> {
    pub fn value(&self, node: Node) -> Value<'_, U> {
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
    pub fn root_value(&self) -> Value<'_, U> {
        let root = self.root();
        self.value(root)
    }

    fn string_value(&self, node: Node) -> Arc<str> {
        let text_id = self.structure.text_id(node.get()).unwrap();
        let text_id = TextId::new(text_id);
        self.text_usage.get_string(text_id)
    }

    fn number_value(&self, node: Node) -> f64 {
        let number_id = self.structure.number_id(node.get()).unwrap();
        self.numbers[number_id]
    }

    fn boolean_value(&self, node: Node) -> bool {
        let boolean_id = self.structure.boolean_id(node.get()).unwrap();
        self.booleans.is_bit_set_unchecked(boolean_id)
    }

    fn array_value(&self, node: Node) -> ArrayValue<'_, U> {
        ArrayValue::new(self, node)
    }

    fn object_value(&self, node: Node) -> ObjectValue<'_, U> {
        ObjectValue::new(self, node)
    }
}

#[cfg(test)]
mod tests {
    use crate::usage::{BitpackingUsageBuilder, UsageBuilder};

    use super::*;

    #[test]
    fn test_number_value() {
        let doc = BitpackingUsageBuilder::parse("42".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Number(42f64));
    }

    #[test]
    fn test_boolean_value_true() {
        let doc = BitpackingUsageBuilder::parse("true".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Boolean(true));
    }

    #[test]
    fn test_boolean_value_false() {
        let doc = BitpackingUsageBuilder::parse("false".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Boolean(false));
    }

    #[test]
    fn test_null_value() {
        let doc = BitpackingUsageBuilder::parse("null".as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::Null);
    }

    #[test]
    fn test_string_value() {
        let doc = BitpackingUsageBuilder::parse(r#""hello""#.as_bytes()).unwrap();
        let v = doc.root_value();
        assert_eq!(v, Value::String("hello".into()));
    }

    #[test]
    fn test_array() {
        let doc = BitpackingUsageBuilder::parse(r#"["a", "b", "c"]"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Array(array_value) = v {
            let mut iter = array_value.into_iter();
            assert_eq!(iter.next(), Some(Value::String("a".into())));
            assert_eq!(iter.next(), Some(Value::String("b".into())));
            assert_eq!(iter.next(), Some(Value::String("c".into())));
            assert_eq!(iter.next(), None);
        } else {
            panic!("Expected an array value");
        }
    }

    #[test]
    fn test_nested_array() {
        let doc = BitpackingUsageBuilder::parse(r#"[1, [2, 3], 4]"#.as_bytes()).unwrap();
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
        let doc =
            BitpackingUsageBuilder::parse(r#"{"key1": "value1", "key2": 42}"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Object(object_value) = v {
            assert_eq!(
                object_value.get("key1"),
                Some(Value::String("value1".into()))
            );
            assert_eq!(object_value.get("key2"), Some(Value::Number(42.0)));
        } else {
            panic!("Expected an object value");
        }
    }

    #[test]
    fn test_object_keys() {
        let doc =
            BitpackingUsageBuilder::parse(r#"{"key1": "value1", "key2": 42}"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Object(object_value) = v {
            let keys: Vec<_> = object_value.keys().collect();
            assert_eq!(keys, vec!["key1", "key2"]);
        } else {
            panic!("Expected an object value");
        }
    }

    #[test]
    fn test_object_values() {
        let doc =
            BitpackingUsageBuilder::parse(r#"{"key1": "value1", "key2": 42}"#.as_bytes()).unwrap();

        let v = doc.root_value();

        if let Value::Object(object_value) = v {
            let values: Vec<_> = object_value.values().collect();
            assert_eq!(
                values,
                vec![Value::String("value1".into()), Value::Number(42.0)]
            );
        } else {
            panic!("Expected an object value");
        }
    }

    #[test]
    fn test_object_entries() {
        let doc =
            BitpackingUsageBuilder::parse(r#"{"key1": "value1", "key2": 42}"#.as_bytes()).unwrap();
        let v = doc.root_value();

        if let Value::Object(object_value) = v {
            let entries: Vec<_> = object_value.iter().collect();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "key1");
            assert_eq!(entries[0].1, Value::String("value1".into()));
            assert_eq!(entries[1].0, "key2");
            assert_eq!(entries[1].1, Value::Number(42.0));
        } else {
            panic!("Expected an object value");
        }
    }
}
