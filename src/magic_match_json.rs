use crate::magic_match::MagicMatch;
use std::error::Error;
use std::io::{BufReader, Read, Seek, BufRead};
use serde_json::{Value, Map};

#[derive(Default, Debug)]
struct MagicJsonMatcher {
    valid_json: bool,
}

impl<S: Seek + Read> MagicMatch<S> for MagicJsonMatcher {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> Result<(), Box<dyn Error>> {
        self.valid_json = false;
        let string = buf.fill_buf()?;
        serde_json::from_slice::<Map<String, Value>>(string)?;
        self.valid_json = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw_bytes::BytesStream;

    #[test]
    fn test() {
        let bytes = "{ \"name\": 10, \"age\": 12 }".as_bytes();
        let stream = BytesStream::new(bytes);
        let mut buff = BufReader::with_capacity(bytes.len(), stream);
        let mut matcher = MagicJsonMatcher::default();
        matcher.magic_match(&mut buff).unwrap();
        assert!(matcher.valid_json);
    }
}
