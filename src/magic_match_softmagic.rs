use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use std::rc::Rc;

use crate::magic_file::MagicFile;
use crate::magic_match::{MagicMatch, MResult};

pub(crate) struct MagicMatcher {
    magic: MagicFile,
    file: Option<Rc<File>>,
}

impl MagicMatcher {
    pub(crate) fn match_file<P>(&mut self, filepath: P)
        where
            P: AsRef<Path>,
    {
        let file = Rc::new(File::open(filepath).unwrap());
        self.file = Some(file.clone());
    }
}

impl<S: Seek + Read> MagicMatch<S> for MagicMatcher {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> MResult {
        Ok(())
    }
}
