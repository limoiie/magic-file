use std::io;
use std::path::Path;
use std::fs::File;
use std::io::BufRead;

use crate::parse_magic_line::MagicLine;
use crate::parse_magic_aux_line::AuxFactor;


#[derive(Debug, Default)]
struct MagicEntry {
    lines: Vec<MagicLine>,
    factor: Option<AuxFactor>,
}

impl MagicEntry {
    fn parse() -> io::Result<()> {
        Ok(())
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
    fn parse(magic_file: &Path) -> io::Result<()> {
        let lines = read_lines(magic_file)?;
        for (line_no, line) in lines.enumerate() {
            let line = line?;
            let chars: Vec<char> = line.chars().collect();
            if !chars.is_empty() {
                match chars[0] {
                    '#' => continue,  // comment line, ignore
                    '!' => {
                        if chars[1] == ':' {
                            // parse_aux_line(&line[2..])
                        }
                    },
                    _ => {
                        println!("line '{}' at {}", line, line_no)
                    }
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}