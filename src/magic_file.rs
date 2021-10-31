use crate::magic_entry::{MagicEntry, MagicEntryBuilder};
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
    pub(crate) entries: Vec<MagicEntry>,
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
            let entry = MagicEntryBuilder::new()
                .parse_until_new_entry(&mut lines_iter)
                .build();
            if let Some(entry) = entry {
                magic.insert_entry(entry);
            }
        }
        magic.sort_by_strength();
        Ok(magic)
    }

    fn insert_entry(&mut self, entry: MagicEntry) {
        self.entries.push(entry)
    }

    fn sort_by_strength(&mut self) {
        self.entries.sort_by_key(MagicEntry::strength);
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
