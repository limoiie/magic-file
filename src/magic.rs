use num_derive::FromPrimitive;
use num_traits::{AsPrimitive, FromPrimitive};
use std::fmt;
use std::rc::Rc;

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

bitflags! {
    pub struct OfsFlags: u16 {
        const NONE     = 0x0000;
        const INDIR    = 0x0001;
        const OFFADD   = 0x0002;
        const IOFFADD  = 0x0004;
        const OP_INDIR = 0x0008;
    }
}

#[repr(u8)]
#[derive(FromPrimitive, Debug, PartialEq, Copy, Clone)]
pub enum ValType {
    Invalid = 0,
    Byte,
    Short,
    Default,
    Long,
    String,
    Date,
    BEShort,
    BELong,
    BEDate,
    LEShort,
    LELong,
    LEDate,
    PString,
    LDate,
    BELDate,
    LELDate,
    Regex,
    BEString16,
    LEString16,
    Search,
    MEDate,
    MELDate,
    MELong,
    Quad,
    LEQuad,
    BEQuad,
    QDate,
    LEQDate,
    BEQDate,
    QLDate,
    LEQLDate,
    BEQLDate,
    Float,
    BEFloat,
    LEFloat,
    Double,
    BEDouble,
    LEDouble,
    BEID3,
    LEID3,
    Indirect,
    QWDate,
    LEQWDate,
    BEQWDate,
    Name,
    Use,
    Clear,
    Der,
    NamesSize,
}

impl Default for ValType {
    fn default() -> Self {
        ValType::Invalid
    }
}

impl ValType {
    pub fn is_string(&self) -> bool {
        match self {
            ValType::String
            | ValType::PString
            | ValType::Regex
            | ValType::BEString16
            | ValType::LEString16
            | ValType::Search
            | ValType::Name
            | ValType::Use
            | ValType::Der
            | ValType::Indirect => true,
            _ => false,
        }
    }

    pub fn is_i8(&self) -> bool {
        *self == ValType::Byte
    }

    pub fn is_i16(&self) -> bool {
        match self {
            ValType::Short => true,
            ValType::BEShort => true,
            ValType::LEShort => true,
            _ => false,
        }
    }

    pub fn is_i32(&self) -> bool {
        match self {
            ValType::Long
            | ValType::Date
            | ValType::BELong
            | ValType::BEDate
            | ValType::LELong
            | ValType::LEDate
            | ValType::LDate
            | ValType::BELDate
            | ValType::LELDate
            | ValType::MEDate
            | ValType::MELDate
            | ValType::MELong
            | ValType::BEID3
            | ValType::LEID3 => true,
            _ => false,
        }
    }

    pub fn is_f32(&self) -> bool {
        match self {
            ValType::Float | ValType::BEFloat | ValType::LEFloat => true,
            _ => false,
        }
    }

    pub fn is_i64(&self) -> bool {
        match self {
            ValType::Quad
            | ValType::LEQuad
            | ValType::BEQuad
            | ValType::QDate
            | ValType::LEQDate
            | ValType::BEQDate
            | ValType::QLDate
            | ValType::LEQLDate
            | ValType::BEQLDate
            | ValType::QWDate
            | ValType::LEQWDate
            | ValType::BEQWDate => true,
            _ => false,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self {
            ValType::Double | ValType::BEDouble | ValType::LEDouble => true,
            _ => false,
        }
    }

    pub fn try_from<T>(c: T) -> Result<ValType, &'static str>
    where
        Self: From<T>,
    {
        let t = Self::from(c);
        if t == ValType::Invalid {
            Err("")
        } else {
            Ok(t)
        }
    }

    fn all() -> Vec<ValType> {
        let mut all: Vec<ValType> = vec![];
        let mut i = 0;
        while let Some(t) = FromPrimitive::from_i32(i) {
            all.push(t);
            i += 1;
        }
        return all;
    }
}

impl From<String> for ValType {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<&str> for ValType {
    fn from(s: &str) -> Self {
        if s.len() == 1 {
            return Self::from(s.chars().nth(0).unwrap());
        } else {
            for i in ValType::all() {
                if s == i.to_string().to_lowercase() {
                    return i;
                }
            }
        }
        ValType::Invalid
    }
}

impl From<char> for ValType {
    fn from(c: char) -> Self {
        match c {
            'l' => ValType::LELong,
            'L' => ValType::BELong,
            'm' => ValType::MELong,
            'h' | 's' => ValType::LEShort,
            'H' | 'S' => ValType::BEShort,
            'c' | 'b' | 'C' | 'B' => ValType::Byte,
            'e' | 'f' | 'g' => ValType::LEDouble,
            'E' | 'F' | 'G' => ValType::BEDouble,
            'i' => ValType::LEID3,
            'I' => ValType::BEID3,
            'q' => ValType::LEQuad,
            'Q' => ValType::BEQuad,
            _ => ValType::Invalid,
        }
    }
}

impl fmt::Display for ValType {
    /// For using obj::to_string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct SignValType {
    pub unsigned: bool,
    pub typ: ValType,
}

impl From<ValType> for SignValType {
    fn from(typ: ValType) -> Self {
        SignValType {
            unsigned: false,
            typ,
        }
    }
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
pub enum Value {
    Str(String),
    Bytes(Vec<u8>),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    pub fn cast_from<T>(typ: SignValType, v: T) -> Value
    where
        T: AsPrimitive<u8>
            + AsPrimitive<u16>
            + AsPrimitive<u32>
            + AsPrimitive<u64>
            + AsPrimitive<i8>
            + AsPrimitive<i16>
            + AsPrimitive<i32>
            + AsPrimitive<i64>
            + AsPrimitive<f32>
            + AsPrimitive<f64>,
    {
        match typ.typ {
            t if t.is_f32() => Value::F32(v.as_()),
            t if t.is_f64() => Value::F64(v.as_()),
            _ => {
                if typ.unsigned {
                    match typ.typ {
                        t if t.is_i8() => Value::U8(v.as_()),
                        t if t.is_i16() => Value::U16(v.as_()),
                        t if t.is_i32() => Value::U32(v.as_()),
                        t if t.is_i64() => Value::U64(v.as_()),
                        _ => Value::U64(v.as_()),
                    }
                } else {
                    match typ.typ {
                        t if t.is_i8() => Value::I8(v.as_()),
                        t if t.is_i16() => Value::I16(v.as_()),
                        t if t.is_i32() => Value::I32(v.as_()),
                        t if t.is_i64() => Value::I64(v.as_()),
                        _ => Value::I64(v.as_()),
                    }
                }
            }
        }
    }

    pub fn cast_to(self, typ: SignValType) -> Value {
        match self {
            Value::Str(_) => self,
            Value::Bytes(_) => self,
            Value::U8(v) => Self::cast_from(typ, v),
            Value::I8(v) => Self::cast_from(typ, v),
            Value::U16(v) => Self::cast_from(typ, v),
            Value::U32(v) => Self::cast_from(typ, v),
            Value::U64(v) => Self::cast_from(typ, v),
            Value::I16(v) => Self::cast_from(typ, v),
            Value::I32(v) => Self::cast_from(typ, v),
            Value::I64(v) => Self::cast_from(typ, v),
            Value::F32(v) => Self::cast_from(typ, v),
            Value::F64(v) => Self::cast_from(typ, v),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::U64(0)
    }
}

#[derive(Debug, PartialEq)]
pub enum UnOperator {
    Absolute,
    Relative,
    Indirect(SignValType),
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
    Val(Value),
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
        Rc::new(Expression::Val(v))
    }
}

impl Default for Expression {
    fn default() -> Self {
        Expression::Val(Value::U64(0))
    }
}
