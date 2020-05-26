use regex::{Regex};

use crate::str_utils;

#[derive(Debug, PartialEq)]
pub(crate) enum FactorOp {
    Noop,
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl Default for FactorOp {
    fn default() -> Self {
        FactorOp::Noop
    }
}

impl From<&str> for FactorOp {
    fn from(s: &str) -> Self {
        match s {
            "+" => FactorOp::Plus,
            "-" => FactorOp::Minus,
            "*" => FactorOp::Multiply,
            "/" => FactorOp::Divide,
            _ => FactorOp::Noop,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct AuxTypes {
    mime: Option<String>,
    apple: Option<String>,
    exts: Vec<String>,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct AuxFactor {
    op: FactorOp,
    val: u32,
}

#[derive(Debug, PartialEq)]
pub(crate) enum AuxInfo {
    Types(AuxTypes),
    Strength(AuxFactor),
}

impl AuxInfo {
    pub(crate) fn parse_aux_line(s: &str) -> AuxInfo {
        let re = Regex::new(r"(?x)
            (?P<t>mime|ext|apple|strength)
            \s*
            (?P<e>.*[^\s])
            \s*$
        ").unwrap();

        if let Some(cap) = re.captures(s) {
            let aux_type = cap.name("t").unwrap().as_str();
            let aux_val = cap.name("e").unwrap().as_str();
            match aux_type {
                "mime" => { Self::parse_line_mime(aux_val) }
                "ext" => { Self::parse_line_ext(aux_val) }
                "apple" => { Self::parse_line_apple(aux_val) }
                "strength" => { Self::parse_line_strength(aux_val) }
                _ => panic!("Unsupport aux type: {}!", aux_type)
            }
        } else {
            panic!("Failed to parse aux line: {}!", s)
        }
    }

    fn parse_line_mime(s: &str) -> AuxInfo {
        AuxInfo::Types(AuxTypes {
            mime: str_utils::ensure_goodchars(s, "+-/.$?:{}"),
            ..AuxTypes::default()
        })
    }

    fn parse_line_ext(s: &str) -> AuxInfo {
        AuxInfo::Types(AuxTypes {
            exts: str_utils::ensure_goodchars(s, ",!+-/@?_$")
                .unwrap().split('/').collect::<Vec<&str>>()
                .iter().map(|&e| e.to_string()).collect(),
            ..AuxTypes::default()
        })
    }

    fn parse_line_apple(s: &str) -> AuxInfo {
        AuxInfo::Types(AuxTypes {
            apple: str_utils::ensure_goodchars(s, "!+-./?"),
            ..AuxTypes::default()
        })
    }

    fn parse_line_strength(s: &str) -> AuxInfo {
        let re = Regex::new(r"(?x)
            (?P<o>[+-/*])
            \s*
            (?P<v>\d+)
        ").unwrap();

        match re.captures(s) {
            Some(cap) => {
                AuxInfo::Strength(AuxFactor {
                    op: cap.name("o").unwrap().as_str().into(),
                    val: cap.name("v").unwrap().as_str().parse::<u32>().unwrap(),
                })
            }
            None => { panic!("Failed to parse strength line: {}!", s) }
        }
    }
}


#[cfg(test)]
mod testcase {
    use super::AuxInfo;
    use crate::parse_magic_aux_line::{FactorOp, AuxTypes, AuxFactor};

    #[test]
    fn test_parse_line_ext() {
        let ext = "txt/doc/pyc";
        let aux = AuxInfo::parse_line_ext(ext);
        assert_eq!(
            AuxInfo::Types(AuxTypes {
                exts: vec!["txt", "doc", "pyc", ].iter()
                    .map(|&s| s)
                    .map(String::from).collect(),
                ..AuxTypes::default()
            }), aux
        );
    }

    #[test]
    fn test_parse_strength() {
        let testcases = vec![
            ("+ 123", AuxInfo::Strength(AuxFactor { op: FactorOp::Plus, val: 123 })),
            ("-567", AuxInfo::Strength(AuxFactor { op: FactorOp::Minus, val: 567 })),
        ];
        for (s, expect) in testcases {
            let aux = AuxInfo::parse_line_strength(s);
            assert_eq!(expect, aux);
        }
    }

    #[test]
    fn test_parse_aux_line() {
        // let mut aux = AuxInfo::default();
        // aux.parse_aux_line("mime  text/x-ruby");
        // aux.parse_aux_line("apple  ????TEXT");
        // aux.parse_aux_line("ext  txt/doc/pyc");
        // assert_eq!(aux.mime.as_ref().unwrap().as_str(), "text/x-ruby");
        // assert_eq!(aux.mime.as_ref().unwrap().as_str(), "text/x-ruby");
        // assert_eq!(aux.apple.as_ref().unwrap().as_str(), "????TEXT");
        // assert_eq!(aux.apple.as_ref().unwrap().as_str(), "????TEXT");
        // assert_eq!(Vec::from_iter(aux.exts.iter().map(String::as_str)), vec![
        //     ".txt", ".doc", ".pyc"
        // ]);
    }
}
