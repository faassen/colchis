use std::io::Write;

use struson::writer::{JsonStreamWriter, JsonWriter};

use crate::usage::UsageIndex;

use super::Document;

impl<U: UsageIndex> Document<U> {
    pub fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        let mut writer = JsonStreamWriter::new(&mut w);

        let root_value = self.root_value();
        root_value.serialize(&mut writer)?;
        writer.finish_document()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::usage::{BitpackingUsageBuilder, RoaringUsageBuilder, UsageBuilder};

    use super::*;

    fn assert_round_trip(input: &str) {
        // parse document from a string
        let doc = BitpackingUsageBuilder::parse(input.as_bytes()).unwrap();
        // serialize to a string
        let mut output = Vec::new();
        doc.serialize(&mut output).unwrap();
        // check that the output is the same as the input
        assert_eq!(String::from_utf8(output).unwrap(), input);
    }

    #[test]
    fn test_round_trip_number() {
        assert_round_trip("42");
    }

    #[test]
    fn test_round_trip_boolean() {
        assert_round_trip("true");
        assert_round_trip("false");
    }

    #[test]
    fn test_round_trip_null() {
        assert_round_trip("null");
    }

    #[test]
    fn test_round_trip_string() {
        assert_round_trip(r#""hello""#);
    }

    #[test]
    fn test_round_trip_array() {
        assert_round_trip(r#"["a","b","c"]"#);
    }

    #[test]
    fn test_round_trip_object() {
        assert_round_trip(r#"{"key1":"value1","key2":"value2"}"#);
    }
}
