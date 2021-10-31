use num_derive::FromPrimitive;
use num_traits::{AsPrimitive, FromPrimitive};
use std::convert::TryInto;
use std::fmt;

pub enum Endian {
    Big,
    Little,
    Median,
    Native,
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
        match self {
            ValType::Byte => true,
            _ => false,
        }
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

    //noinspection DuplicatedCode
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

    //noinspection DuplicatedCode
    pub fn is_be(&self) -> bool {
        match self {
            ValType::BEShort
            | ValType::BELong
            | ValType::BEDate
            | ValType::BELDate
            | ValType::BEString16
            | ValType::BEQuad
            | ValType::BEQDate
            | ValType::BEQLDate
            | ValType::BEFloat
            | ValType::BEDouble
            | ValType::BEID3
            | ValType::BEQWDate => true,
            _ => false,
        }
    }

    //noinspection DuplicatedCode
    pub fn is_le(&self) -> bool {
        match self {
            ValType::LEShort
            | ValType::LELong
            | ValType::LEDate
            | ValType::LELDate
            | ValType::LEString16
            | ValType::LEQuad
            | ValType::LEQDate
            | ValType::LEQLDate
            | ValType::LEFloat
            | ValType::LEDouble
            | ValType::LEID3
            | ValType::LEQWDate => true,
            _ => false,
        }
    }

    pub fn is_me(&self) -> bool {
        match self {
            ValType::MEDate | ValType::MELDate | ValType::MELong => true,
            _ => false,
        }
    }

    pub fn is_ne(&self) -> bool {
        !self.is_be() && !self.is_me() && !self.is_le()
    }

    pub fn endian(&self) -> Endian {
        if self.is_be() {
            Endian::Big
        } else if self.is_le() {
            Endian::Little
        } else if self.is_me() {
            Endian::Median
        } else {
            Endian::Native
        }
    }

    pub fn size(&self) -> Option<usize> {
        if self.is_i8() {
            Some(1)
        } else if self.is_i16() {
            Some(2)
        } else if self.is_i32() || self.is_f32() {
            Some(4)
        } else if self.is_i64() || self.is_f64() {
            Some(8)
        } else {
            None
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

impl From<&Value> for ValType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Bytes(_) => ValType::String,
            Value::I8(_) | Value::U8(_) => ValType::Byte,
            Value::I16(_) | Value::U16(_) => ValType::Short,
            Value::I32(_) | Value::U32(_) => ValType::Long,
            Value::I64(_) | Value::U64(_) => ValType::Quad,
            Value::F32(_) => ValType::Float,
            Value::F64(_) => ValType::Double,
        }
    }
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct SignValType {
    pub unsigned: bool,
    pub typ: ValType,
}

impl SignValType {
    pub fn new(unsigned: bool, typ: ValType) -> Self {
        SignValType { unsigned, typ }
    }
}

impl From<ValType> for SignValType {
    fn from(typ: ValType) -> Self {
        SignValType::new(false, typ)
    }
}

impl From<&Value> for SignValType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Bytes(_) => SignValType::new(true, v.into()),
            Value::U8(_) => SignValType::new(true, v.into()),
            Value::U16(_) => SignValType::new(true, v.into()),
            Value::U32(_) => SignValType::new(true, v.into()),
            Value::U64(_) => SignValType::new(true, v.into()),
            Value::I8(_) => SignValType::new(false, v.into()),
            Value::I16(_) => SignValType::new(false, v.into()),
            Value::I32(_) => SignValType::new(false, v.into()),
            Value::I64(_) => SignValType::new(false, v.into()),
            Value::F32(_) => SignValType::new(false, v.into()),
            Value::F64(_) => SignValType::new(false, v.into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
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

impl Default for Value {
    fn default() -> Self {
        Value::I64(0)
    }
}

impl Value {
    pub fn cast_from<T>(typ: &SignValType, v: &T) -> Value
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

    //noinspection DuplicatedCode
    pub fn cast_from_bytes(typ: &SignValType, v: &Vec<u8>) -> Value {
        match typ.typ {
            ValType::Invalid
            | ValType::Default
            | ValType::String
            | ValType::PString
            | ValType::Regex
            | ValType::BEString16
            | ValType::LEString16
            | ValType::Search
            | ValType::Name
            | ValType::Use
            | ValType::Indirect
            | ValType::Clear
            | ValType::Der
            | ValType::NamesSize => Default::default(),
            _ => Self::cast_from_bytes_to_num(typ, v),
        }
    }

    pub fn cast_from_bytes_to_num_<T>(unsigned: bool, endian: Endian, v: &Vec<u8>) -> Value {
        Value::I64(0)
    }

    pub fn cast_from_bytes_to_num(typ: &SignValType, v: &Vec<u8>) -> Value {
        if typ.typ.is_f32() {
            return Value::F32(f32::from_be_bytes(v[..4].try_into().unwrap()));
        } else if typ.typ.is_f64() {
            return Value::F64(f64::from_be_bytes(v[..8].try_into().unwrap()));
        } else if typ.unsigned {
            if typ.typ.is_be() {
                if typ.typ.is_i8() {
                    return Value::U8(u8::from_be_bytes(v[..1].try_into().unwrap()));
                } else if typ.typ.is_i16() {
                    return Value::U16(u16::from_be_bytes(v[..2].try_into().unwrap()));
                } else if typ.typ.is_i32() {
                    return Value::U32(u32::from_be_bytes(v[..4].try_into().unwrap()));
                } else if typ.typ.is_i64() {
                    return Value::U64(u64::from_be_bytes(v[..8].try_into().unwrap()));
                }
            } else if typ.typ.is_me() {
                if typ.typ.is_i32() {
                    return Value::U32(u32::from_le_bytes([v[2], v[3], v[0], v[1]]));
                }
            } else if typ.typ.is_le() {
                if typ.typ.is_i8() {
                    return Value::U8(u8::from_le_bytes(v[..1].try_into().unwrap()));
                } else if typ.typ.is_i16() {
                    return Value::U16(u16::from_le_bytes(v[..2].try_into().unwrap()));
                } else if typ.typ.is_i32() {
                    return Value::U32(u32::from_le_bytes(v[..4].try_into().unwrap()));
                } else if typ.typ.is_i64() {
                    return Value::U64(u64::from_le_bytes(v[..8].try_into().unwrap()));
                }
            }
        } else {
            if typ.typ.is_be() {
                if typ.typ.is_i8() {
                    return Value::I8(i8::from_be_bytes(v[..1].try_into().unwrap()));
                } else if typ.typ.is_i16() {
                    return Value::I16(i16::from_be_bytes(v[..2].try_into().unwrap()));
                } else if typ.typ.is_i32() {
                    return Value::I32(i32::from_be_bytes(v[..4].try_into().unwrap()));
                } else if typ.typ.is_i64() {
                    return Value::I64(i64::from_be_bytes(v[..8].try_into().unwrap()));
                }
            } else if typ.typ.is_me() {
                if typ.typ.is_i32() {
                    return Value::I32(i32::from_le_bytes([v[2], v[3], v[0], v[1]]));
                }
            } else if typ.typ.is_le() {
                if typ.typ.is_i8() {
                    return Value::I8(i8::from_le_bytes(v[..1].try_into().unwrap()));
                } else if typ.typ.is_i16() {
                    return Value::I16(i16::from_le_bytes(v[..2].try_into().unwrap()));
                } else if typ.typ.is_i32() {
                    return Value::I32(i32::from_le_bytes(v[..4].try_into().unwrap()));
                } else if typ.typ.is_i64() {
                    return Value::I64(i64::from_le_bytes(v[..8].try_into().unwrap()));
                }
            }
        }
        panic!("Unsupport cast from raw bytes to this type!")
    }

    pub fn cast_by(&self, typ: &SignValType) -> Value {
        match self {
            Value::Bytes(v) => Value::Bytes(v.clone()),
            Value::U8(v) => Self::cast_from(typ, v),
            Value::U16(v) => Self::cast_from(typ, v),
            Value::U32(v) => Self::cast_from(typ, v),
            Value::U64(v) => Self::cast_from(typ, v),
            Value::I8(v) => Self::cast_from(typ, v),
            Value::I16(v) => Self::cast_from(typ, v),
            Value::I32(v) => Self::cast_from(typ, v),
            Value::I64(v) => Self::cast_from(typ, v),
            Value::F32(v) => Self::cast_from(typ, v),
            Value::F64(v) => Self::cast_from(typ, v),
        }
    }

    pub fn as_type_of(&mut self, o: &Value) {
        self.cast_by(&o.into());
    }

    pub fn str_len(&self) -> Option<usize> {
        match self {
            Value::Bytes(b) => Some(b.len()),
            _ => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Value::Bytes(bytes) => bytes.len(),
            Value::I8(_) | Value::U8(_) => 1,
            Value::I16(_) | Value::U16(_) => 2,
            Value::I32(_) | Value::U32(_) | Value::F32(_) => 4,
            Value::I64(_) | Value::U64(_) | Value::F64(_) => 8,
        }
    }

    fn order(&self) -> usize {
        match self {
            Value::Bytes(_) => 10,
            Value::U8(_) => 0,
            Value::I8(_) => 1,
            Value::U16(_) => 2,
            Value::I16(_) => 3,
            Value::U32(_) => 4,
            Value::I32(_) => 5,
            Value::U64(_) => 6,
            Value::I64(_) => 7,
            Value::F32(_) => 8,
            Value::F64(_) => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_by() {
        let testcases = [
            (
                SignValType::new(true, ValType::Short),
                Value::I64(10),
                Value::U16(10),
            ),
            (
                SignValType::new(true, ValType::Byte),
                Value::I64(1024),
                Value::U8(0),
            ),
        ];
        for (typ, val, expect) in testcases.iter() {
            assert_eq!(&val.cast_by(typ), expect)
        }
    }
}
