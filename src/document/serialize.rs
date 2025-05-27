use std::io::Write;

use struson::writer::{JsonStreamWriter, JsonWriter};

use super::Document;

impl Document {
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
    use super::*;

    #[test]
    fn test_round_trip_number() {
        let input = "42";
        // parse document from a string
        let doc = Document::parse(input.as_bytes()).unwrap();
        // serialize to a string
        let mut output = Vec::new();
        doc.serialize(&mut output).unwrap();
        // check that the output is the same as the input
        assert_eq!(String::from_utf8(output).unwrap(), input);
    }
}
