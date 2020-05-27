use std::io;
use std::path::Path;
use std::fs::File;
use std::io::BufRead;

use crate::parse_magic_line::MagicLine;
use crate::parse_magic_aux_line::{AuxFactor, AuxInfo};
use std::fmt::{Debug, Display};
use log::info;
use std::iter::Peekable;


#[derive(Debug, Default)]
struct MagicEntry {
    lines: Vec<MagicLine>,
    factor: Option<AuxFactor>,
}

impl MagicEntry {
    fn parse<P>(&mut self, iter: &mut Peekable<P>)
        where P: Iterator<Item=io::Result<String>> {
        while let Some(line_res) = iter.peek() {
            if let Ok(line) = line_res {
                if self.meet_new_entry(line.as_str()) {
                    return
                }
                self.parse_line(line.as_str());
            } else {
                // println!("{}:{}:{}", magic_file, no, line.unwrap_err());
            }
            iter.next();
        }
    }

    fn meet_new_entry(&self, line: &str) -> bool {
        !self.lines.is_empty() && MagicLine::is_entry_line(line)
    }

    fn parse_line(&mut self, line: &str) {
        println!("parsing {}...", line);

        let mut chars = line.chars();
        match chars.next() {
            None => return,
            Some('#') => return,
            Some('!') => {
                if let Some(':') = chars.next() {
                    match AuxInfo::parse_aux_line(&line[2..]) {
                        AuxInfo::Types(types) => {
                            self.lines.last_mut().unwrap().aux = Some(types)
                        }
                        AuxInfo::Strength(factor) => {
                            self.factor = Some(factor)
                        }
                    }
                }
            }
            _ => {
                let magic_line =
                    MagicLine::parse_entry_line(line);
                self.lines.push(magic_line);
            }
        }
    }
}


fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}


struct MagicFile {
    entries: Vec<MagicEntry>,
}

impl MagicFile {
    fn parse<P>(magic_file: P) -> io::Result<()>
        where P: AsRef<Path> + Display, {
        let mut lines_iter =
            read_lines(&magic_file)?
                .into_iter()
                .peekable();

        while lines_iter.peek().is_some() {
            println!("\n=================================");
            let mut me = MagicEntry::default();
            me.parse(&mut lines_iter);
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::parse_magic_entry::MagicFile;
    #[test]
    fn test() {
        MagicFile::parse("/usr/share/file/magic/database");
    }
}
