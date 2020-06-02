use std::fs::{File, Metadata};
use std::io::BufReader;

use crate::magic_match::{MagicMatch, MResult};

#[derive(Default, Debug)]
struct MagicFileSystemMatcher {
    metadata: Option<Metadata>,
}

impl MagicMatch<File> for MagicFileSystemMatcher {
    fn magic_match(&mut self, buf: &mut BufReader<File>) -> MResult {
        self.metadata = Some(buf.get_ref().metadata()?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let path = "/Users/ligengwang/Downloads/nmt_inspired.zip";
        let file = File::open(path).unwrap();
        let mut buff = BufReader::new(file);
        let mut matcher = MagicFileSystemMatcher::default();
        matcher.magic_match(&mut buff).unwrap();
        println!("{:?}", matcher);
    }
}
