use std::iter::FromIterator;
use std::rc::Rc;

use bitflags::_core::iter::Peekable;
use bitflags::_core::str::Chars;

use crate::magic::{Action, Expression, MaskFlags, Operator, SignValType, ValType, Value};
use crate::magic_line::MagicLine;
use num_traits::Num;

impl MagicLine {
    pub(crate) fn parse_line2(line: &str) -> Self {
        // println!("parsing {}", line);

        let mut ml = Self::default();

        let mut i = 0 as usize;
        let mut chars: Vec<char> = line.chars().collect();

        ml.parse_cont(&mut i, &mut chars);
        Self::jump_blank(&mut i, &mut chars);

        ml.parse_ofst(&mut i, &mut chars);
        Self::jump_blank(&mut i, &mut chars);

        ml.parse_cmp_typ(&mut i, &mut chars);
        Self::jump_blank(&mut i, &mut chars);

        ml.parse_desc(&mut i, &mut chars);
        return ml;
    }

    fn parse_cont(&mut self, i: &mut usize, chars: &mut Vec<char>) {
        let mut l = 0;
        while let Some(&c) = chars.get(*i) {
            if c != '>' {
                break;
            }
            l += 1;
            *i += 1
        }
        self.cont_lvl = l
    }

    fn parse_ofst(&mut self, i: &mut usize, chars: &mut Vec<char>) {
        let mut rel = false;
        let mut indir = false;
        let mut rel_indir = false;
        let mut off_indir = false;

        if *chars.get(*i).unwrap() == '&' {
            *i += 1;
            rel = true
        }
        if *chars.get(*i).unwrap() == '(' {
            *i += 1;
            indir = true;
            if rel {
                rel = false;
                rel_indir = true
            }
        }
        if *chars.get(*i).unwrap() == '&' {
            *i += 1;
            rel = true
        }

        let u = Self::get_digit(i, chars, false);
        let typ = Self::parse_ofs_typ(i, chars);

        let mut action = None;
        let op = Self::get_operator(i, chars);
        if op != Operator::Noop {
            *i += 1;

            if *chars.get(*i).unwrap() == '(' {
                *i += 1;
                off_indir = true;
            }
            let off_in_val = Self::get_digit(i, chars, false);

            if off_indir {
                assert_eq!(*chars.get(*i).unwrap(), ')');
                *i += 1;
            }

            let val = if off_indir {
                Expression::unop_indir(None, Expression::val(off_in_val), None)
            } else {
                Expression::val(off_in_val)
            };
            action = Some(Action::Num { op, val })
        }

        if indir {
            let c = *chars.get(*i).unwrap();
            assert_eq!(c, ')');
            *i += 1;
        }

        let mut e = Expression::val(u);
        if rel {
            e = Expression::unop_rel(e, None);
        }
        if indir {
            e = Expression::unop_indir(Some(typ), e, action);
            if rel_indir {
                e = Expression::unop_rel(e, None);
            }
        }
        self.exp = e
    }

    fn parse_ofs_typ(i: &mut usize, chars: &mut Vec<char>) -> SignValType {
        let c = *chars.get(*i).unwrap();
        if c == '.' || c == ',' {
            *i += 2;
            let unsigned = c == '.';
            let t = *chars.get(*i).unwrap();
            let typ = ValType::from(t);
            return SignValType { unsigned, typ };
        }
        return SignValType {
            unsigned: false,
            typ: Default::default(),
        };
    }

    fn parse_cmp_typ(&mut self, i: &mut usize, chars: &mut Vec<char>) {
        let start = *i;
        while let Some(&c) = chars.get(*i) {
            match c {
                '0'..='9' | 'a'..='z' => *i += 1,
                _ => break,
            }
        }

        let end = *i;
        let mut ts = String::from_iter(&chars[start..end]);

        let mut unsigned = false;
        let &c = chars.get(start).unwrap();
        if c == 'u' && ts.len() != 3 {
            unsigned = true;
            ts = ts[1..].to_string();
        }

        let typ = ValType::from(ts.as_str());
        // println!("main val type: {} {:?}", unsigned, typ);

        let sign_typ = SignValType { unsigned, typ };
        let mask = Self::parse_mask(i, chars, &typ);
        Self::jump_blank(i, chars);
        let reln = Self::parse_reln(i, chars, &typ);

        self.exp = Expression::unop_indir(Some(sign_typ), self.exp.clone(), mask);
        self.exp = Expression::unop_abs(self.exp.clone(), reln);
    }

    fn parse_mask(i: &mut usize, chars: &mut Vec<char>, typ: &ValType) -> Option<Action> {
        let mut action = None;
        let op = Self::get_operator(i, chars);
        if op != Operator::Noop {
            *i += 1;

            if typ.is_string() {
                let mut range = 0;
                let mut flags = MaskFlags::NONE;
                while let Some(&c) = chars.get(*i) {
                    match c {
                        '/' => {
                            *i += 1;
                        }
                        '0'..='9' => {
                            let d = Self::get_digit(i, chars, false);
                            match d {
                                Value::U64(v) => range = v,
                                _ => range = 0,
                            }
                        }
                        c if Self::is_blank(c) => {
                            break;
                        }
                        c => {
                            flags |= MaskFlags::from(c);
                            *i += 1;
                        }
                    }
                }
                action = Some(Action::Str { flags, range })
            } else {
                let off_in_val = Self::get_digit(i, chars, false);
                let val = Expression::val(off_in_val);
                action = Some(Action::Num { op, val })
            }
        }
        return action;
    }

    fn parse_reln(i: &mut usize, chars: &mut Vec<char>, typ: &ValType) -> Option<Action> {
        if let Some(&c) = chars.get(*i) {
            match chars.get(*i + 1) {
                None => return None,
                Some(&d) => {
                    if c == 'x' && Self::is_blank(d) {
                        *i += 1;
                        return None
                    }
                },
            }
        }

        let mut action = None;
        let op = Self::get_operator(i, chars);

        let op = if op == Operator::Noop {
            Operator::EQ
        } else {
            *i += 1;
            Self::jump_blank(i, chars);
            op
        };

        if typ.is_string() {
            let start = *i;
            while let Some(&c) = chars.get(*i) {
                match c {
                    '\\' => {
                        *i += 1;
                        match *chars.get(*i).unwrap() {
                            't' => '\t',
                            '0' => '\0',
                            'n' => '\n',
                            'r' => '\r',
                            'v' => '\u{b}',
                            'a' => '\u{7}',
                            'b' => '\u{8}',
                            'f' => '\u{c}',
                            unk => unk,
                        };
                        *i += 1;
                    }
                    c if Self::is_blank(c) => {
                        break;
                    }
                    _ => {
                        *i += 1;
                    }
                }
            }
            let end = *i;
            let s = String::from_iter(&chars[start..end]);
            action = Some(Action::Num {
                op,
                val: Expression::val(Value::Str(s)),
            })
        } else {
            let off_in_val = Self::get_digit(i, chars, false);
            let val = Expression::val(off_in_val);
            action = Some(Action::Num { op, val })
        }
        return action;
    }

    fn parse_desc(&mut self, i: &mut usize, chars: &mut Vec<char>) {
        let start = *i;
        while let Some(&c) = chars.get(*i) {
            match c {
                '|' => {
                    break;
                },
                _ => {
                    *i += 1;
                }
            }
        }
        let end = *i;
        self.desc = String::from_iter(&chars[start..end]);
    }

    fn jump_blank(i: &mut usize, chars: &mut Vec<char>) {
        while let Some(&c) = chars.get(*i) {
            if !Self::is_blank(c) {
                break;
            }
            *i += 1
        }
    }

    fn get_digit(i: &mut usize, chars: &mut Vec<char>, should_f: bool) -> Value {
        let mut sign = true;
        let c = *chars.get(*i).unwrap();
        if c == '+' {
            *i += 1;
            sign = true
        } else if c == '-' {
            *i += 1;
            sign = false
        }

        if *chars.get(*i).unwrap() == '0' {
            match chars.get(*i + 1) {
                Some(&'x') => {
                    *i += 2;
                    let start = *i;
                    while let Some(&c) = chars.get(*i) {
                        match c {
                            '0'..='9' | 'a'..='f' | 'A'..='F' => {
                                *i += 1;
                            }
                            _ => { break }
                        }
                    }

                    let end = *i;
                    let s = String::from_iter(&chars[start..end]);
                    let u = u64::from_str_radix(s.as_str(), 16).unwrap();
                    return Value::U64(u);
                },
                _ => {}
            }
        }

        let start = *i;
        let mut f = false;
        while let Some(&c) = chars.get(*i) {
            match c {
                '0'..='9' => {
                    *i += 1;
                }
                '.' => {
                    if should_f {
                        f = true;
                        *i += 1;
                    } else {
                        break;
                    }
                }
                _ => break
            }
        }
        let end = *i;
        let s = String::from_iter(&chars[start..end]);
        if f {
            let x = f64::from_str_radix(s.as_str(), 10).unwrap();
            return Value::F64(x);
        }
        let u = u64::from_str_radix(s.as_str(), 10).unwrap();
        return Value::U64(u);
    }

    fn get_operator(i: &mut usize, chars: &mut Vec<char>) -> Operator {
        let c = *chars.get(*i).unwrap();
        Operator::from(c as u8)
    }

    fn is_blank(c: char) -> bool {
        c == ' ' || c == '\0' || c == '\t'
    }

    fn default() -> Self {
        MagicLine {
            cont_lvl: 0,
            exp: Rc::new(Default::default()),
            desc: "".to_string(),
            typ_code: 0,
            aux: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::parse_magic_entry;

    #[test]
    fn test() {
        let cases = vec![
            ">>>>>>>>>>>>>>&-1	string		>\0		1st record",
            ">>>&(&10.l-(2)) ubelong%0x123 <10 this is a good|123",
            ">>>&(&10.l-(2)) string/w/cW/20 asdf this is a good|123",
        ];
        for s in cases {
            println!("parsing {}...", s);
            let ml = MagicLine::parse_line2(s);
            println!("magic line: {:?}", ml)
        }
    }

}
