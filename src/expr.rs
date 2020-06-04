use std::rc::Rc;

use crate::expr_value::{Value, SignValType};

bitflags! {
    pub struct MaskFlags: u16 {
        const NONE                            = 0x0000;
        const COMPACT_WHITESPACE              = 0x0001;
        const COMPACT_OPTIONAL_WHITESPACE     = 0x0002;
        const IGNORE_LOWERCASE                = 0x0004;
        const IGNORE_UPPERCASE                = 0x0008;
        const REGEX_OFFSET_START              = 0x0010;
        const BINTEST                         = 0x0020;
        const TEXTTEST                        = 0x0040;
        const TRIM                            = 0x0080;
        const PSTRING_1_LE                    = 0x0100;
        const PSTRING_2_BE                    = 0x0200;
        const PSTRING_2_LE                    = 0x0400;
        const PSTRING_4_BE                    = 0x0800;
        const PSTRING_4_LE                    = 0x1000;
        const PSTRING_LENGTH_INCLUDES_ITSELF  = 0x2000;
        const INDIRECT_RRELATIVE              = 0x4000;
        const PSTRING_LEN = Self::PSTRING_1_LE.bits | Self::PSTRING_2_BE.bits |
            Self::PSTRING_2_LE.bits | Self::PSTRING_4_BE.bits | Self::PSTRING_4_LE.bits;
    }
}

static ALL_IN_CHARS: &str = "WwcCsbtTBHhLlJr";

impl MaskFlags {
    fn all_in_chars() -> &'static str {
        return ALL_IN_CHARS;
    }

    pub fn is_pstring(&self) -> bool {
        return self.contains(
            MaskFlags::PSTRING_1_LE
                | MaskFlags::PSTRING_2_BE
                | MaskFlags::PSTRING_2_LE
                | MaskFlags::PSTRING_4_BE
                | MaskFlags::PSTRING_4_LE,
        );
    }

    pub fn try_from(c: char) -> Result<MaskFlags, &'static str> {
        let t = Self::from(c);
        if t == MaskFlags::NONE {
            Err("")
        } else {
            Ok(t)
        }
    }
}

impl From<u16> for MaskFlags {
    fn from(bits: u16) -> Self {
        return MaskFlags::from_bits_truncate(bits);
    }
}

impl From<String> for MaskFlags {
    fn from(s: String) -> Self {
        MaskFlags::from(s.as_str())
    }
}

impl From<&str> for MaskFlags {
    fn from(s: &str) -> Self {
        if s.len() == 1 {
            let all = MaskFlags::all_in_chars();
            if let Some(i) = all.find(s) {
                return MaskFlags::from(1 << i as u16);
            }
        }
        MaskFlags::NONE
    }
}

impl From<char> for MaskFlags {
    fn from(c: char) -> Self {
        MaskFlags::from(c.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum UnOperator {
    Absolute,
    Relative,
    Indirect(SignValType),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Noop,
    // logical
    And,
    Or,
    Xor,
    Not,
    // comparison
    EQ,
    LT,
    GT,
    // computation
    Add,
    Minus,
    Multiply,
    Divide,
    Modulo,
}

impl Default for Operator {
    fn default() -> Self {
        Operator::Noop
    }
}

impl From<&str> for Operator {
    fn from(s: &str) -> Self {
        Self::from(*s.as_bytes().first().unwrap())
    }
}

impl From<u8> for Operator {
    fn from(b: u8) -> Self {
        use Operator::*;
        match b as char {
            '&' => And,
            '|' => Or,
            '^' => Xor,
            '!' => Xor,
            '+' => Add,
            '-' => Minus,
            '*' => Multiply,
            '/' => Divide,
            '%' => Modulo,
            '=' => EQ,
            '<' => LT,
            '>' => GT,
            _ => Operator::Noop,
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum Action {
    Num { op: Operator, val: Rc<Expression> },
    Str { flags: MaskFlags, range: u64 },
}

impl Action {
    pub fn update_range(self, range: u64) -> Action {
        match self {
            Action::Str {
                flags,
                range: _range,
            } => Action::Str { flags, range },
            _ => self,
        }
    }

    pub fn update_flags(self, new_flags: MaskFlags) -> Action {
        match self {
            Action::Str { flags, range } => Action::Str {
                flags: flags | new_flags,
                range,
            },
            _ => self,
        }
    }

    pub fn default_str() -> Self {
        Action::Str {
            flags: MaskFlags::NONE,
            range: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    UnOp(UnOperator, Rc<Expression>, Option<Action>),
    Val(Rc<Value>),
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Val(Rc::new(Value::U64(0)))
    }
}

impl Expression {
    pub fn unop_indir(
        typ: Option<SignValType>,
        exp: Rc<Expression>,
        mask: Option<Action>,
    ) -> Rc<Expression> {
        let typ = typ.unwrap_or(Default::default());
        Rc::new(Expression::UnOp(UnOperator::Indirect(typ), exp, mask))
    }

    pub fn unop_rel(exp: Rc<Expression>, mask: Option<Action>) -> Rc<Expression> {
        Rc::new(Expression::UnOp(UnOperator::Relative, exp, mask))
    }

    pub fn unop_abs(exp: Rc<Expression>, mask: Option<Action>) -> Rc<Expression> {
        if mask.is_none() {
            exp
        } else {
            Rc::new(Expression::UnOp(UnOperator::Absolute, exp, mask))
        }
    }

    pub fn val(v: Value) -> Rc<Expression> {
        Rc::new(Expression::Val(Rc::new(v)))
    }
}
