use std::fmt::Debug;
use std::io;
use std::iter::Peekable;

use crate::magic::{AuxFactor, AuxLine};
use crate::magic_line::{Aux, MagicLine};

#[derive(Debug, Default)]
pub(crate) struct MagicEntry {
    lines: Vec<MagicLine>,
    factor: Option<AuxFactor>,
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
            Some('!') if Some(':') == chars.next() => self.handle_aux_line(&line[2..]),
            _ => self.handle_magic_line(line),
        }
    }

    fn handle_magic_line(&mut self, line: &str) {
        self.lines.push(MagicLine::parse_line(line))
    }

    fn handle_aux_line(&mut self, line: &str) {
        match Aux::parse_line(line) {
            AuxLine::Type(typ) => {
                self.lines.last_mut().unwrap().attach_aux(typ);
            }
            AuxLine::Strength(factor) => self.factor = Some(factor),
        }
    }
}
