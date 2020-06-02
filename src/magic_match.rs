use crate::magic_file::MagicFile;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use std::rc::Rc;
use std::error::Error;

pub type MResult = Result<(), Box<dyn Error>>;

const TRUNK_BYTES: usize = 1024 * 1024;

pub(crate) struct MagicMatcher {
    pub(crate) magic_file: MagicFile,
    pub(crate) file: Option<Rc<File>>,
}

pub(crate) trait MagicMatch<S: Seek + Read> {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> MResult;
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
