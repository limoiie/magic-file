use regex::{Regex, Match};

use crate::magic::{CmpType, MaskOp, StrModifier, RelnOp};
use crate::parse_magic_aux_line::AuxInfo;


#[derive(Debug, PartialEq)]
enum Mask {
    Num { op: MaskOp, val: u64 },
    Str { flags: StrModifier, range: u64 },
}

impl Default for Mask {
    fn default() -> Self {
        Mask::Num { op: MaskOp::default(), val: 0 }
    }
}


#[derive(Debug, Default)]
pub(crate) struct MagicLine {
    cont_lvl: usize,
    typ_code: u32,
    cmp_type: CmpType,
    cmp_unsigned: bool,
    mask: Mask,
    reln_op: RelnOp,
    reln_val: u64,
    aux: Option<AuxInfo>,
    desc: String,
}


impl MagicLine {
    fn line_regex() -> Regex {
        // todo!("support parse the string contains escaped characters");
        Regex::new(r"(?x)
            (?P<c>>*)         # continue level
            (?P<o>[^\s]+)     # offset expression
            \s+
            (?P<t>\w+)                 # type of comparison
            (?P<m>[+\-*/&|^%][^\s]+)?  # mask expression
            \s+
            (?P<r>[^\s]+)     # relation expression
            \s+
            (?P<d>[^|]*)      # description
            \|?
            (?P<n>\d*)?     # type code
        ").unwrap()
    }

    pub(crate) fn is_entry_line(line: &str) -> bool {
        let re = Self::line_regex();
        if let Some(cap) = re.captures(line) {
            let cont = cap.name("c");
            return cont.is_some() && cont.unwrap().end() == 0
        }
        false
    }

    pub(crate) fn parse_entry_line(&mut self, s: &str) {
        let re = Self::line_regex();
        if let Some(cap) = re.captures(s) {
            self.parse_cont_part(cap.name("c"));
            self.parse_ofst_part(cap.name("o"));
            self.parse_type_part(cap.name("t"));
            self.parse_mask_part(cap.name("m"));
            self.parse_reln_part(cap.name("r"));
            self.parse_desc_part(cap.name("d"));
            self.parse_code_part(cap.name("n"));
        };
    }

    fn parse_cont_part(&mut self, s: Option<Match>) {
        self.cont_lvl = s.unwrap().as_str().len();
    }

    fn parse_ofst_part(&mut self, s: Option<Match>) {
        println!("ofst: {}", s.unwrap().as_str());
    }

    fn parse_type_part(&mut self, s: Option<Match>) {
        let s = s.unwrap().as_str();
        self.cmp_unsigned = s.starts_with("u");
        self.cmp_type =
            if self.cmp_unsigned {
                s[1..].into()
            } else {
                s.into()
            };
        //    todo: parse as an SUS type if invalid
        //    todo: parse as def|name|use if still invalid
    }

    fn parse_mask_part(&mut self, s: Option<Match>) {
        if let Some(o) = s {
            let s = o.as_str();
            let (op, modifier) = s.split_at(1);

            self.mask =
                if self.cmp_type.is_string() {
                    self.parse_str_modifier(op, modifier)
                } else {
                    self.parse_num_modifier(op, modifier)
                }
        }
    }

    fn parse_str_modifier(&self, _op: &str, modifier: &str) -> Mask {
        let re = Regex::new(r"(?P<r>\d*)(?P<m>.*)").unwrap();
        if let Some(cap) = re.captures(modifier) {
            let range = cap.name("r").unwrap().as_str().parse::<u64>().unwrap_or(0);
            let modifier = cap.name("m").unwrap().as_str();
            let flags = self.parse_chars_modifier(modifier);
            return Mask::Str { flags, range };
        }
        panic!("Failed to parse str modifier, invalid modifier: {}!", modifier)
    }

    fn parse_chars_modifier(&self, modifier: &str) -> StrModifier {
        // remained modifier is a string like `w/c/W/'
        let mut flags = StrModifier::NONE;
        let chars: Vec<&str> = modifier.split('/').collect();
        for c in chars {
            if c.is_empty() { continue; }

            let flag = StrModifier::from(c);
            if flag.is_pstring() {
                // assert!(self.cmp_type == CmpType::PString ||
                //     self.cmp_type == CmpType::Regex);
                match flag {
                    StrModifier::PSTRING_LENGTH_INCLUDES_ITSELF => {}
                    _ => {
                        // only one type pstring can be enabled at the same time
                        flags.remove(StrModifier::PSTRING_LEN);
                    }
                };
            };
            flags.insert(flag);
        }

        flags
    }

    fn parse_num_modifier(&self, op: &str, modifier: &str) -> Mask {
        // note: the subfix type decorator is ignored
        let re = Regex::new(r"(\d+)u?.?").unwrap();
        if let Some(cap) = re.captures(modifier) {
            let op = MaskOp::from(op);
            let val = cap.get(0).unwrap().as_str().parse::<u64>().unwrap();
            return Mask::Num { op, val };
        }
        panic!("Failed to parse num modifier, invalid num: {}!", modifier)
    }

    fn parse_reln_part(&mut self, s: Option<Match>) {
        // note: `=' behind &, ^, = is ignored
        let s = s.unwrap().as_str();
        // todo!("support parse the string contains escapsed characters");
        let re = Regex::new(r"(?P<x>x)|(?P<r>[><^&=!])=?(?P<n>\d*)").unwrap();
        if let Some(cap) = re.captures(s) {
            if cap.name("x").is_some() {
                self.reln_op = RelnOp::Eq
            } else {
                let reln_op = cap.name("r").unwrap().as_str();
                self.reln_op = RelnOp::from(reln_op);
            }
        }
    }

    fn parse_desc_part(&mut self, s: Option<Match>) {
        let s = s.unwrap().as_str();
        let re = Regex::new(r"(?P<b>\\b)?(?P<d>.*)").unwrap();
        if let Some(cap) = re.captures(s) {
            let no_whitespace = cap.name("b").is_some();
            self.desc = cap.name("d").unwrap().as_str().to_string();
            if !no_whitespace {
                self.desc.insert(0, ' ')
            }
        }
    }

    fn parse_code_part(&mut self, s: Option<Match>) {
        self.typ_code = s.unwrap().as_str().parse::<u32>().unwrap_or(0);
    }
}


#[cfg(test)]
mod tests {
    use regex::Regex;
    use super::MagicLine;
    use crate::magic::{CmpType, StrModifier};
    use crate::parse_magic_line::Mask;

    #[test]
    fn test_is_entry_line() {
        let testcases = vec![
            (">>>>&9	ulelong	x	attributes 0x%x", false),
            ("9	ulelong	x	attributes 0x%x", true),
            ("&9	ulelong	x	attributes 0x%x", true),
        ];

        for (s, expect) in testcases {
            assert_eq!(MagicLine::is_entry_line(s), expect);
        }
    }

    #[test]
    fn test_parse_str_modifiers() {
        let re = Regex::new(r"(?P<r>\d*)(?P<m>[^\d]*)\b").unwrap();
        if let Some(cap) = re.captures(r"c\W") {
            let a = cap.name("r").unwrap().as_str();
            let b = cap.name("m").unwrap().as_str();
            println!("a={}, b={}", a, b);
        }
    }

    #[test]
    fn test_magic_parse_entry_line() {
        let mut m = MagicLine::default();

        m.parse_entry_line(r">>>>&9	ulelong	x	attributes 0x%x");
        assert_eq!(m.cmp_type, CmpType::LELong);
        assert_eq!(m.cmp_unsigned, true);
        assert_eq!(m.desc, " attributes 0x%x");

        m.parse_entry_line(r"0	lestring16	x	attributes 0x%x|123");
        assert_eq!(m.cmp_type, CmpType::LEString16);
        assert_eq!(m.cmp_unsigned, false);
        assert_eq!(m.desc, " attributes 0x%x");
        assert_eq!(m.typ_code, 123);

        m.parse_entry_line(r">8	lestring16/c/W	x	\b, attributes 0x%x");
        assert_eq!(m.cmp_type, CmpType::LEString16);
        assert_eq!(m.cmp_unsigned, false);
        match m.mask {
            Mask::Num { .. } => {}
            Mask::Str { flags, range } => {
                println!("flags: {}, range: {}", flags.bits(), range);
                assert!(flags.contains(StrModifier::IGNORE_LOWERCASE));
                assert!(flags.contains(StrModifier::COMPACT_WHITESPACE));
            }
        }
        assert_eq!(m.desc, ", attributes 0x%x");
    }
}
