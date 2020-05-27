use regex::{Regex, Match};

use crate::magic::{CmpType, MaskOp, StrModifier, RelnOp};
use crate::parse_magic_aux_line::AuxTypes;


#[derive(Debug, PartialEq)]
pub(crate) enum Mask {
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
    pub cont_lvl: usize,
    pub typ_code: u32,
    pub cmp_type: CmpType,
    pub cmp_unsigned: bool,
    pub mask: Mask,
    pub reln_op: RelnOp,
    pub reln_val: u64,
    pub aux: Option<AuxTypes>,
    pub desc: String,
}


lazy_static! {
    // todo!("support parse the string contains escaped characters");
    static ref RE_LINE: Regex = {
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
    };

    static ref RE_DESC: Regex = {
        Regex::new(r"(?x)
            (?P<b>\\b)?
            (?P<d>.*)
        ").unwrap()
    };

    // todo!("support parse the string contains escapsed characters");
    static ref RE_RELN: Regex = {
        Regex::new(r"(?x)
            (?P<x>x)|
            (?P<r>[><^&=!])
            =?
            (?P<n>\d*)
        ").unwrap()
    };

    static ref RE_NUMX: Regex = {
        Regex::new(r"(?x)
            (?P<d>
                (\d+)|
                (0x[0-9a-fA-F]+)|
                (0b[01]+)
            )u?.?
        ").unwrap()
    };

    static ref RE_ENTRY: Regex = {
        Regex::new(r"^\s*[0-9(&]").unwrap()
    };

    static ref RE_STR: Regex = {
        Regex::new(r"(?x)
            (?P<r>\d*)
            (?P<m>.*)
        ").unwrap()
    };

}

impl MagicLine {

    pub(crate) fn is_entry_line(line: &str) -> bool {
        // entry line is the line start with an offset expression
        RE_ENTRY.is_match(line)
    }

    pub(crate) fn parse_entry_line(s: &str) -> MagicLine {
        let mut magic_line = Self::default();
        if let Some(cap) = RE_LINE.captures(s) {
            magic_line.parse_cont_part(cap.name("c"));
            magic_line.parse_ofst_part(cap.name("o"));
            magic_line.parse_type_part(cap.name("t"));
            magic_line.parse_mask_part(cap.name("m"));
            magic_line.parse_reln_part(cap.name("r"));
            magic_line.parse_desc_part(cap.name("d"));
            magic_line.parse_code_part(cap.name("n"));
        }
        magic_line
    }

    fn parse_cont_part(&mut self, s: Option<Match>) {
        self.cont_lvl = s.unwrap().as_str().len();
    }

    fn parse_ofst_part(&mut self, s: Option<Match>) {
        // println!("ofst: {}", s.unwrap().as_str());
        //    todo: parse offset
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
        if let Some(cap) = RE_STR.captures(modifier) {
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
        if let Some(cap) = RE_NUMX.captures(modifier) {
            let op = MaskOp::from(op);
            let val = cap.name("d").unwrap().as_str();
            let val = u64::from_str_radix(val, 2)
                .or(u64::from_str_radix(val, 10))
                .or(u64::from_str_radix(val, 16))
                .unwrap();
            return Mask::Num { op, val };
        }
        panic!("Failed to parse num modifier, invalid num: {}!", modifier)
    }

    fn parse_reln_part(&mut self, s: Option<Match>) {
        // note: `=' behind &, ^, = is ignored
        let s = s.unwrap().as_str();
        if let Some(cap) = RE_RELN.captures(s) {
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
        if let Some(cap) = RE_DESC.captures(s) {
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


impl MagicLine {

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
            ("# extracted from header/code files by Graeme Wilford", false),
        ];

        for (s, expect) in testcases {
            println!("comparing {}...", s);
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
        let m =
            MagicLine::parse_entry_line(r">>>>&9	ulelong	x	attributes 0x%x");
        assert_eq!(m.cmp_type, CmpType::LELong);
        assert_eq!(m.cmp_unsigned, true);
        assert_eq!(m.desc, " attributes 0x%x");

        let m =
            MagicLine::parse_entry_line(r"0	lestring16	x	attributes 0x%x|123");
        assert_eq!(m.cmp_type, CmpType::LEString16);
        assert_eq!(m.cmp_unsigned, false);
        assert_eq!(m.desc, " attributes 0x%x");
        assert_eq!(m.typ_code, 123);

        let m =
            MagicLine::parse_entry_line(r">8	lestring16/c/W	x	\b, attributes 0x%x");
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
