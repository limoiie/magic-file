use std::io;
use std::path::Path;
use std::fs::File;
use std::io::BufRead;
use std::fmt::{Debug, Display};
use std::iter::Peekable;

use crate::parse_magic_aux_line::{AuxFactor, AuxInfo};
use crate::magic_line::MagicLine;


#[derive(Debug, Default)]
struct MagicEntry {
    lines: Vec<MagicLine>,
    factor: Option<AuxFactor>,
}

impl MagicEntry {
    fn parse<P>(&mut self, lines: &mut Peekable<P>)
        where P: Iterator<Item=io::Result<String>> {
        while let Some(line_res) = lines.peek() {
            if let Ok(line) = line_res {
                if self.meet_new_entry(line.as_str()) {
                    return
                }
                self.parse_line(line.as_str());
            }
            lines.next();
        }
    }

    fn meet_new_entry(&self, line: &str) -> bool {
        !self.lines.is_empty() && MagicLine::is_entry_line(line)
    }

    fn parse_line(&mut self, line: &str) {
        // println!("parsing {}...", line);

        let mut chars = line.chars();
        match chars.next() {
            None => return,
            Some('#') => return,
            Some('!') => {
                if let Some(':') = chars.next() {
                //     match AuxInfo::parse_aux_line(&line[2..]) {
                //         AuxInfo::Types(types) => {
                //             self.lines.last_mut().unwrap().aux = Some(types)
                //         }
                //         AuxInfo::Strength(factor) => {
                //             self.factor = Some(factor)
                //         }
                //     }
                }
            }
            _ => {
                self.lines.push(MagicLine::parse_line(line));
            }
        }
    }
}


fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}


pub(crate) struct MagicFile {
    entries: Vec<MagicEntry>,
}

impl MagicFile {
    pub(crate) fn parse<P>(magic_file: P) -> io::Result<()>
        where P: AsRef<Path>, {
        let mut lines_iter =
            read_lines(&magic_file)?
                .into_iter()
                .peekable();

        while lines_iter.peek().is_some() {
            // println!("\n=================================");
            let mut me = MagicEntry::default();
            me.parse(&mut lines_iter);
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::fs;
    use crate::parse_magic_entry::MagicFile;
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
