use crate::magic_entry::MagicEntry;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Default, Debug)]
pub(crate) struct MagicFile {
    entries: Vec<MagicEntry>,
}

impl MagicFile {
    pub(crate) fn parse<P>(magic_file: P) -> io::Result<MagicFile>
    where
        P: AsRef<Path>,
    {
        let mut magic = MagicFile::default();

        let mut lines_iter = read_lines(&magic_file)?.into_iter().peekable();
        while lines_iter.peek().is_some() {
            // println!("\n=================================");
            magic.entries.push(MagicEntry::parse_entry(&mut lines_iter));
        }
        Ok(magic)
    }
}

#[cfg(test)]
mod tests {
    use crate::magic_file::MagicFile;
    use std::fs;

    #[test]
    fn test_onefile() {
        MagicFile::parse("/usr/share/file/magic/database");
    }

    #[test]
    fn test_files() {
        for f in fs::read_dir("/usr/share/file/magic").unwrap() {
            let p = f.unwrap().path();
            println!("parsing {:?}...", p);
            MagicFile::parse(p);
        }
    }
}
