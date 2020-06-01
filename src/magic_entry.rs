use std::fmt::Debug;
use std::io;
use std::iter::Peekable;

use crate::magic_line::{MagicLine, AuxLine, AuxStrength, AuxType};

#[derive(Debug, Default)]
pub(crate) struct MagicEntry {
    lines: Vec<MagicLine>,
    factor: Option<AuxStrength>,
}

impl MagicEntry {
    pub(crate) fn parse<P>(&mut self, lines: &mut Peekable<P>)
    where
        P: Iterator<Item = io::Result<String>>,
    {
        while let Some(line_res) = lines.peek() {
            if let Ok(line) = line_res {
                if self.meet_new_entry(line.as_str()) {
                    return;
                }
                self.digest_line(line.as_str());
            }
            lines.next();
        }
    }

    fn meet_new_entry(&self, line: &str) -> bool {
        !self.lines.is_empty() && MagicLine::is_entry_line(line)
    }

    fn digest_line(&mut self, line: &str) {
        // println!("parsing {}...", line);

        let mut chars = line.chars();
        match chars.next() {
            // blank line
            None => return,
            // comment line
            Some('#') => return,
            // aux line, which contains either a type or a strength
            Some('!') if Some(':') == chars.next() => {
                match AuxLine::parse_line(&line[2..]) {
                    AuxLine::Type(typ) => {
                        self.attach_aux_type_to_last_line(typ)
                    }
                    AuxLine::Strength(factor) => {
                        self.attach_strength_to_entry(factor)
                    }
                }
            }
            // otherwise, must be a magic line
            _ => {
                self.lines.push(MagicLine::parse_line(line))
            }
        }
    }

    fn attach_aux_type_to_last_line(&mut self, typ: AuxType) {
        self.lines.last_mut().unwrap().attach_aux(typ)
    }

    fn attach_strength_to_entry(&mut self, factor: AuxStrength) {
        self.factor = Some(factor)
    }

}
