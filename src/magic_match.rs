use std::io::{BufReader, Read, Seek};
use std::error::Error;

pub type MResult = Result<(), Box<dyn Error>>;

const TRUNK_BYTES: usize = 1024 * 1024;

pub(crate) trait MagicMatch<S: Seek + Read> {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> MResult;
}
