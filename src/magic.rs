use num_traits::FromPrimitive;
use num_derive::FromPrimitive;
use std::fmt;

bitflags! {
    struct MagicFlags: u8 {
        const INDIR        = 0b00000001;  /* if '(...)' appears */
        const OFFADD       = 0b00000010;  /* if '>&' or '>...(&' appears */
        const INDIROFFADD  = 0b00000100;  /* if '>&(' appears */
        const UNSIGNED     = 0b00001000;  /* comparison is unsigned */
        const NOSPACE      = 0b00010000;  /* suppress space character before output */
        const BINTEST      = 0b00100000;  /* test is for a binary type (only top-level tests) */
        const TEXTTEST     = 0b01000000;  /* for passing to file_softmagic */
    }
}

impl MagicFlags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

impl From<u8> for MagicFlags {
    fn from(bits: u8) -> Self {
        return MagicFlags::from_bits_truncate(bits);
    }
}


bitflags! {
    pub(crate) struct StrModifier: u16 {
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

impl StrModifier {
    fn all_in_chars() -> &'static str {
        const ALL_IN_CHARS: &str = "WwcCsbtTBHhLlJr";
        return ALL_IN_CHARS;
    }

    pub(crate) fn is_pstring(&self) -> bool {
        return self.contains(
            StrModifier::PSTRING_1_LE |
                StrModifier::PSTRING_2_BE |
                StrModifier::PSTRING_2_LE |
                StrModifier::PSTRING_4_BE |
                StrModifier::PSTRING_4_LE
        );
    }
}

impl From<u16> for StrModifier {
    fn from(bits: u16) -> Self {
        return StrModifier::from_bits_truncate(bits);
    }
}

impl From<String> for StrModifier {
    fn from(s: String) -> Self {
        if s.len() == 1 {
            let all = StrModifier::all_in_chars();
            if let Some(i) = all.find(s.as_str()) {
                return StrModifier::from(1 << i as u16);
            }
        }
        panic!("Failed to parse str modifier: {}!", s)
    }
}

impl From<&str> for StrModifier {
    fn from(s: &str) -> Self {
        StrModifier::from(s.to_string())
    }
}

impl From<char> for StrModifier {
    fn from(c: char) -> Self {
        StrModifier::from(c.to_string())
    }
}


#[repr(u8)]
#[derive(FromPrimitive, Debug, PartialEq)]
pub(crate) enum CmpType {
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

impl Default for CmpType {
    fn default() -> Self { CmpType::Invalid }
}

impl CmpType {
    pub(crate) fn is_string(&self) -> bool {
        match self {
            CmpType::String |
            CmpType::PString |
            CmpType::Regex |
            CmpType::BEString16 |
            CmpType::LEString16 |
            CmpType::Search |
            CmpType::Name |
            CmpType::Use |
            CmpType::Indirect => true,
            _ => false
        }
    }

    fn all() -> Vec<CmpType> {
        let mut all: Vec<CmpType> = vec![];
        let mut i = 0;
        while let Some(t) = FromPrimitive::from_i32(i) {
            all.push(t);
            i += 1;
        }
        return all;
    }
}

impl From<String> for CmpType {
    fn from(s: String) -> Self {
        let s = s.to_lowercase();
        for i in CmpType::all() {
            let o = i.to_string();
            if s == o.to_lowercase() {
                return i;
            }
        }
        CmpType::Invalid
    }
}

impl From<&str> for CmpType {
    fn from(s: &str) -> Self {
        CmpType::from(s.to_string())
    }
}

impl fmt::Display for CmpType {
    /// For using obj::to_string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Debug, PartialEq)]
pub(crate) enum MaskOp {
    Noop,
    And,
    Or,
    Xor,
    Add,
    Minus,
    Multiply,
    Divide,
    Modulo,
}

impl Default for MaskOp {
    fn default() -> Self { MaskOp::Noop }
}

impl From<&str> for MaskOp {
    fn from(s: &str) -> Self {
        match s {
            "&" => MaskOp::And,
            "|" => MaskOp::Or,
            "^" => MaskOp::Xor,
            "+" => MaskOp::Add,
            "-" => MaskOp::Minus,
            "*" => MaskOp::Multiply,
            "/" => MaskOp::Divide,
            "%" => MaskOp::Modulo,
            _ => MaskOp::Noop,
        }
    }
}


#[derive(Debug, PartialEq)]
pub(crate) enum RelnOp {
    Noop,
    And,
    Xor,
    Eq,
    Less,
    Greater,
    Not,
}

impl Default for RelnOp {
    fn default() -> Self { RelnOp::Noop }
}

impl From<&str> for RelnOp {
    fn from(s: &str) -> Self {
        match s {
            "&" => RelnOp::And,
            "^" => RelnOp::Xor,
            "=" => RelnOp::Eq,
            "<" => RelnOp::Less,
            ">" => RelnOp::Greater,
            "!" => RelnOp::Not,
            _ => RelnOp::Noop,
        }
    }
}

enum RelnVal {

}


enum CondType {
    None = 0,
    If = 1,
    ElIf = 2,
    Else = 3,
}


#[cfg(test)]
mod tests {
    use super::MagicFlags;
    use crate::magic::CmpType;

    #[test]
    fn test_main() {
        let y = MagicFlags::from(10u8);
        let x = MagicFlags::INDIR;
        println!("{:?}, {:?}", x, y);
        println!("{:?}", MagicFlags::all())
    }

    #[test]
    fn test_from_i32_to_cmp_typ() {
        let x = CmpType::all();
        for i in x {
            println!("{}", i.to_string())
        }
    }

    #[test]
    fn test_from_string_to_cmp_typ() {
        let testcases = vec![
            ("use", CmpType::Use),
            ("invalid", CmpType::Invalid),
        ];
        for (name, typ) in testcases {
            assert_eq!(CmpType::from(name.to_string()), typ)
        }
    }
}