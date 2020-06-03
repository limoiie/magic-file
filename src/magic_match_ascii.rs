use std::io::{BufRead, BufReader, Read, Seek};

use crate::magic_match::{MResult, MagicMatch};

#[derive(Debug, PartialEq)]
enum AsciiType {
    Binary,
    Utf7,
    Utf8 { with_bom: bool },
    Utf16 { big_end: bool },
    Ascii,
    AsciiLatin1,
    AsciiExtended,
}

impl Default for AsciiType {
    fn default() -> Self {
        AsciiType::Binary
    }
}

#[derive(Default, Debug, PartialEq)]
struct MagicAsciiMatcher {
    typ: AsciiType,
}

impl<S: Seek + Read> MagicMatch<S> for MagicAsciiMatcher {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> MResult {
        let str = buf.fill_buf().unwrap();
        self.typ = Self::match_with_ascii(str)
            .or(Self::match_with_latin1(str))
            .or(Self::match_with_extended(str))
            .or(Self::match_with_utf7(str))
            .or(Self::match_with_utf8(str))
            .or(Self::match_with_utf16(str))
            .unwrap_or(AsciiType::Binary);
        Ok(())
    }
}

const F: u8 = 0; /* character never appears in text */
const T: u8 = 1; /* character appears in plain ASCII text */
const I: u8 = 2; /* character appears in ISO-8859 text */
const X: u8 = 3; /* character appears in non-ISO extended ASCII (Mac, IBM PC) */

const CHARS: [u8; 256] = [
    /*                  BEL BS HT LF VT FF CR    */
    F, F, F, F, F, F, F, T, T, T, T, T, T, T, F, F, /* 0x0X */
    /*                              ESC          */
    F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, /* 0x1X */
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, /* 0x2X */
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, /* 0x3X */
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, /* 0x4X */
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, /* 0x5X */
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, /* 0x6X */
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, F, /* 0x7X */
    /*            NEL                            */
    X, X, X, X, X, T, X, X, X, X, X, X, X, X, X, X, /* 0x8X */
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, /* 0x9X */
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, /* 0xaX */
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, /* 0xbX */
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, /* 0xcX */
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, /* 0xdX */
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, /* 0xeX */
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, /* 0xfX */
];

use AsciiType::*;

impl MagicAsciiMatcher {
    fn match_with_ascii(str: &[u8]) -> Option<AsciiType> {
        for &c in str {
            if c != T {
                return None;
            }
        }
        Some(Ascii)
    }

    fn match_with_latin1(str: &[u8]) -> Option<AsciiType> {
        for &c in str {
            if c != T && c != I {
                return None;
            }
        }
        Some(AsciiLatin1)
    }

    fn match_with_extended(str: &[u8]) -> Option<AsciiType> {
        for &c in str {
            if c != T && c != I && c != X {
                return None;
            }
        }
        Some(AsciiExtended)
    }

    fn match_with_utf7(str: &[u8]) -> Option<AsciiType> {
        if str.len() > 4 && &str[..3] == b"+/v".as_ref() {
            match str[3] {
                b'8' | b'9' | b'+' | b'/' => return Some(Utf7),
                _ => {}
            }
        }
        None
    }

    fn match_with_utf8(str: &[u8]) -> Option<AsciiType> {
        if std::str::from_utf8(str).is_ok() {
            Some(Utf8 { with_bom: false })
        } else {
            let (bom, str) = str.split_at(3);
            if bom == b"BOM".as_ref() && std::str::from_utf8(str).is_ok() {
                Some(Utf8 { with_bom: true })
            } else {
                None
            }
        }
    }

    fn match_with_utf16(str: &[u8]) -> Option<AsciiType> {
        unsafe {
            if (str.len() & 1) == 0 {
                let len = str.len() / 2;
                let data = str.as_ptr() as *const u16;
                let raw = core::slice::from_raw_parts(data, len);
                if String::from_utf16(raw).is_ok() {
                    return Some(Utf16 { big_end: raw[0] == 0xFEFF })
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        println!("hello, world");
    }
}
