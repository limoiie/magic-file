use std::fmt::Debug;
use std::io;
use std::iter::Peekable;

use crate::magic::{Action, Expression, Operator, ValType};
use crate::magic_line::{AuxLine, AuxStrength, AuxType, MagicLine};
use std::cmp::max;

/// Magic entry to provide a sequence of check rules
///
/// A magic entry, which consists a list of magic lines, starts
/// from a magic line with continue-level 0 and ends at the start
/// of another magic entry.
#[derive(Debug, Default)]
pub(crate) struct MagicEntry {
    /// a sequence of rule lines
    lines: Vec<MagicLine>,
    /// represents for the priority of entry
    factor: Option<AuxStrength>,
}

const STRENGTH_UNIT: i32 = 10;

impl MagicEntry {
    pub(crate) fn parse_entry<P>(lines: &mut Peekable<P>) -> MagicEntry
    where
        P: Iterator<Item = io::Result<String>>,
    {
        let mut entry = MagicEntry::default();
        while let Some(line_res) = lines.peek() {
            if let Ok(line) = line_res {
                if entry.meet_new_entry(line.as_str()) {
                    break;
                }
                entry.digest_line(line.as_str());
            }
            lines.next();
        }
        entry
    }

    fn meet_new_entry(&self, line: &str) -> bool {
        !self.lines.is_empty() && MagicLine::is_entry_line(line)
    }

    fn digest_line(&mut self, line: &str) {
        println!("parsing {}...", line);

        let mut chars = line.chars();
        match chars.next() {
            // blank line
            None => return,
            // comment line
            Some('#') => return,
            // aux line, which contains either a type or a strength
            Some('!') if Some(':') == chars.next() => match AuxLine::parse_line(&line[2..]) {
                AuxLine::Type(typ) => self.attach_aux_type_to_last_line(typ),
                AuxLine::Strength(factor) => self.attach_strength_to_entry(factor),
            },
            // otherwise, must be a magic line
            _ => self.lines.push(MagicLine::parse_line(line)),
        }
    }

    fn attach_aux_type_to_last_line(&mut self, typ: AuxType) {
        self.lines.last_mut().unwrap().attach_aux(typ)
    }

    fn attach_strength_to_entry(&mut self, factor: AuxStrength) {
        self.factor = Some(factor)
    }

    /// Get the priority of this entry
    ///
    /// The priority is used to sort all the entries so that the interested
    /// or the cheap rules can be applied first.
    pub(crate) fn strength(&self) -> i32 {
        let mut strength = STRENGTH_UNIT * 2;
        if let Some(magic_line) = self.lines.get(0) {
            strength += self.strength_delta_from_cmp_val().unwrap_or(0);
            strength += self.strength_delta_from_cmp_op().unwrap_or(0);
        }
        self.strength_by_factor(strength).unwrap_or(1)
    }

    fn strength_delta_from_cmp_val(&self) -> Option<i32> {
        let magic_line = self.top_line()?;
        let fn_str_len = || Some(magic_line.reln_val()?.str_len()? as i32);
        Some(match magic_line.typ.typ {
            typ if typ.is_i8() => STRENGTH_UNIT,
            typ if typ.is_i16() => STRENGTH_UNIT * 2,
            typ if typ.is_i32() => STRENGTH_UNIT * 4,
            typ if typ.is_i64() => STRENGTH_UNIT * 8,
            ValType::PString | ValType::String => STRENGTH_UNIT * fn_str_len()?,
            ValType::LEString16 | ValType::BEString16 => STRENGTH_UNIT * fn_str_len()? / 2,
            ValType::Search => max(STRENGTH_UNIT, fn_str_len()?),
            ValType::Der | ValType::Regex => STRENGTH_UNIT,
            _ => 0,
        })
    }

    fn strength_delta_from_cmp_op(&self) -> Option<i32> {
        let magic_line = self.top_line()?;
        Some(match magic_line.reln_op()? {
            Operator::EQ => STRENGTH_UNIT,
            Operator::LT | Operator::GT => -STRENGTH_UNIT * 2,
            Operator::And | Operator::Xor => -STRENGTH_UNIT,
            _ => 0,
        })
    }

    fn strength_by_factor(&self, strength: i32) -> Option<i32> {
        let AuxStrength { op, val } = self.factor.as_ref()?;
        Some(match op {
            Operator::Add => strength + val,
            Operator::Minus => strength - val,
            Operator::Multiply => strength * val,
            Operator::Divide => strength / val,
            _ => 0,
        })
    }

    fn top_line(&self) -> Option<&MagicLine> {
        self.lines.get(0)
    }
}
