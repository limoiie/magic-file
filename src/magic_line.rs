use regex::Regex;
use std::rc::Rc;
use std::str;
use std::str::FromStr;

use crate::magic::*;

use crate::magic::Expression as Expr;

#[derive(Debug, PartialEq)]
pub enum AuxType {
    Mime(String),
    Apple(String),
    Exts(Vec<String>),
}

#[derive(Debug, Default, PartialEq)]
pub struct AuxTypes {
    mime: Option<String>,
    apple: Option<String>,
    exts: Vec<String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct AuxStrength {
    pub op: Operator,
    pub val: u32,
}

#[derive(Debug, PartialEq)]
pub enum AuxLine {
    Type(AuxType),
    Strength(AuxStrength),
}


impl AuxLine {
    pub(crate) fn parse_line(line: &str) -> AuxLine {
        magic_line::aux(line).unwrap()
    }
}

#[derive(Debug, Default)]
pub struct MagicLine {
    pub cont_lvl: u32,
    pub exp: Rc<Expr>,
    pub desc: String,
    pub typ_code: u32,
    pub aux: Option<AuxTypes>,
}

lazy_static! {
    static ref RE_ENTRY: Regex = { Regex::new(r"^\s*[0-9(&]").unwrap() };
}

impl MagicLine {
    pub(crate) fn is_entry_line(line: &str) -> bool {
        RE_ENTRY.is_match(line)
    }

    pub(crate) fn parse_line(line: &str) -> Self {
        magic_line::line(line).unwrap()
    }

    pub(crate) fn attach_aux(&mut self, aux_typ: AuxType) {
        if self.aux.is_none() {
            self.aux = Some(Default::default())
        }
        let aux = self.aux.as_mut().unwrap();
        match aux_typ {
            AuxType::Mime(m) => aux.mime = Some(m),
            AuxType::Apple(a) => aux.apple = Some(a),
            AuxType::Exts(e) => aux.exts = e,
        }
    }
}

peg::parser! {
  grammar magic_line() for str {
    // line ===============================================
    // >>>[offset] [type][mask?] [reln] [desc]|?[type_code]
    pub rule line() -> MagicLine
      = cont_lvl:cont() __?                      // >>>
          ofs:ofs_exp() __                       // [offset]
          typ:cmp_sign_typ() mask:mask(typ)? __  // [typ][mask?]
          reln:reln(typ) __?                     // [reln]
          desc:desc() ['|']?                     // [desc]|?
          typ_code:typ_code()                    // [type_code]
      {
        let exp = Expr::unop_abs(
          Expr::unop_indir(Some(typ), ofs, mask), reln);
        MagicLine { cont_lvl, exp, desc, typ_code, aux: None }
      }
    // ====================================================

    // continue level =====================================
    pub rule cont() -> u32 = c:['>']* { c.len() as u32 }
    // ====================================================

    // offset exp =========================================
    pub rule ofs_exp() -> Rc<Expr> = ofs_exp_binop()
    pub rule ofs_exp_() -> Rc<Expr> = ofs_exp_unop() / ofs_exp_val()

    pub rule ofs_exp_binop() -> Rc<Expr>
      = e:ofs_exp_() m:mask_num()? { Expr::unop_abs(e, m) }

    pub rule ofs_exp_unop() -> Rc<Expr>
      = "(" e:ofs_exp_() t:ofs_sign_typ()? m:ofs_act()? ")" { Expr::unop_indir(t, e, m) }
      / "&" e:ofs_exp_() { Expr::unop_rel(e, None) }

    pub rule ofs_exp_val() -> Rc<Expr> = v:value() { Expr::val(v) }

    pub rule ofs_act() -> Action = op:val_op() val:ofs_exp() { Action::Num { op, val } }
    // ====================================================

    // type ===============================================
    rule ofs_sign_typ() -> SignValType
      = ['.'] typ:ofs_type() { SignValType { unsigned: true,  typ } }
      / [','] typ:ofs_type() { SignValType { unsigned: false, typ } }

    rule cmp_sign_typ() -> SignValType
      = ['u'] typ:cmp_type() { SignValType { unsigned: true,  typ } }
      / eps() typ:cmp_type() { SignValType { unsigned: false, typ } }

    rule cmp_type() -> ValType = s:ident() {? ValType::try_from(s) }
    rule ofs_type() -> ValType = c:any_char() {? ValType::try_from(c) }

    rule assert_str(typ: SignValType, is_str: bool) = eps() {?
        if is_str == typ.typ.is_string() { Ok(()) } else { Err("") }
      }

    rule switch_typ<T>(typ: SignValType, s_rule: rule<T>, n_rule: rule<T>) -> T
      = assert_str(typ, true)  x:s_rule() { x }
      / assert_str(typ, false) x:n_rule() { x }
    // ====================================================

    // mask action ========================================
    pub rule mask(typ: SignValType) -> Action
      = switch_typ(typ, <mask_str()>, <mask_num()>)

    pub rule mask_num() -> Action = m:mask_num_() _typ() { m }
    pub rule mask_str() -> Action = mask_sep() m:mask_str_() { m }

    rule mask_num_() -> Action
      = op:val_op() v:value() { Action::Num { op, val: Expr::val(v) } }

    rule mask_str_() -> Action
      = range:mask_str_range() mask_sep()? mask:mask_str_() { mask.update_range(range) }
      / flags:mask_str_flags() mask_sep()? mask:mask_str_() { mask.update_flags(flags) }
      / eps() { Action::default_str() }

    rule mask_str_range() -> u64 = digit_u64()
    rule mask_str_flags() -> MaskFlags = c:any_char() {? MaskFlags::try_from(c) }

    rule mask_sep() = ['/']
    // ====================================================

    // reln action ========================================
    pub rule reln(typ: SignValType) -> Option<Action>
      = switch_typ(typ, <reln_str()>, <reln_num()>)

    pub rule reln_str() -> Option<Action> = o:reln_(<reln_str_val()>) { o }
    pub rule reln_num() -> Option<Action> = o:reln_(<reln_num_val()>) { o }

    pub rule reln_str_val() -> Value = value_raw()
    pub rule reln_num_val() -> Value = value()

    rule reln_(reln_val: rule<Value>) -> Option<Action>
      = reln_noop() { None }
      / o:reln_op()? __? val:reln_val() {
        Some(Action::Num {
          op: o.unwrap_or(Operator::EQ),
          val: Expr::val(val),
        })
      }

    rule reln_noop() = ['x'] peek_whitespace()
    rule reln_op() -> Operator = cmp_op()
    // ====================================================

    // desc ===============================================
    pub rule desc() -> String
      = backspace() s:desc_content() { s.to_string() }
      / s:desc_content() { " ".to_owned() + s }

    pub rule desc_content() -> &'input str = $( (!['|'] [_])* )
    // ====================================================

    // type code ==========================================
    pub rule typ_code() -> u32 = digit_u32() / eps() { 0 }
    // ====================================================


    // aux line ===========================================
    pub rule aux() -> AuxLine
      = "mime"     __ m:mime()     comment()? { m }
      / "apple"    __ a:apple()    comment()? { a }
      / "ext"      __ e:ext()      comment()? { e }
      / "strength" __ s:strength() comment()? { s }

    pub rule mime() -> AuxLine
      = m:$(['0'..='9' | 'a'..='z' | 'A'..='Z' | '+' | '-' | '*' |
             '/' | '.' | '$' | '?' | ':' | '{' | '}']*) {
        AuxLine::Type(AuxType::Mime(m.to_string()))
      }

    pub rule apple() -> AuxLine
      = a:$(['0'..='9' | 'a'..='z' | 'A'..='Z' | '+' | '-' | '.' |
             '/' | '!' | '?']*) {
        AuxLine::Type(AuxType::Apple(a.to_string()))
      }

    pub rule ext() -> AuxLine
      = e:$(['0'..='9' | 'a'..='z' | 'A'..='Z' | '+' | '-' | ',' | '/' |
             '!' | '@' | '?' | '_' | '$']*) {
        AuxLine::Type(AuxType::Exts(
          e.split('/').collect::<Vec<&str>>()
            .iter().map(|&e| e.to_string()).collect()))
      }

    pub rule strength() -> AuxLine
      = op:strength_op() __? val:digit_u32() {
        AuxLine::Strength(AuxStrength { op, val })
      }

    pub rule comment() = __ ['#'] [_]*
    // ====================================================


    // ====================================================
    //
    // util rules
    //
    // ====================================================

    // val op =============================================
    pub rule val_op() -> Operator
      = c:$( ['&' | '|' | '^' | '+' | '-' | '*' | '/' | '%'] ) { c.into() }

    pub rule cmp_op() -> Operator
      = c:$( ['&' | '^' | '=' | '<' | '>' | '!'] ) { c.into() }

    pub rule strength_op() -> Operator
      = c:$( ['+' | '-' | '*' | '/'] ) { c.into() }
    // ====================================================

    // Value ==============================================
    pub rule value() -> Value
      = value_f64()
      / value_u64()
      / value_i64()

    pub rule value_u64() -> Value = d:digit_u64() { Value::U64(d) }
    pub rule value_i64() -> Value = d:digit_i64() { Value::I64(d) }
    pub rule value_f64() -> Value = f:float_f64() { Value::F64(f) }
    pub rule value_raw() -> Value = s:raw_bytes() { Value::Bytes(s) }
    // ====================================================

    // float ==============================================
    pub rule float_f32() -> f32 = c:float() { f32::from_str(c).unwrap() }
    pub rule float_f64() -> f64 = c:float() { f64::from_str(c).unwrap() }

    pub rule float() -> &'input str
      = c:$(['+' | '-']? ['0'..='9']+ "." ['0'..='9']+ ("e" ['+' | '-'] ['0'..='9']+)?)
    // ====================================================

    // int ================================================
    pub rule digit_u32() -> u32
      = s:hex() { u32::from_str_radix(s, 16).unwrap() }
      // / s:bin() { u32::from_str_radix(s, 02).unwrap() }
      / s:dec() { u32::from_str_radix(s, 10).unwrap() }

    pub rule digit_i64() -> i64
      = ['-'] v:digit_u64() { -(v as i64) }
      / ['+']? v:digit_u64() { (v as i64) }

    pub rule digit_u64() -> u64
      = s:hex() { u64::from_str_radix(s, 16).unwrap() }
      // / s:bin() { u64::from_str_radix(s, 02).unwrap() }
      / s:dec() { u64::from_str_radix(s, 10).unwrap() }

    rule dec() -> &'input str = $(['0'..='9']+)
    rule hex() -> &'input str = "0x" s:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) { s }
    // rule bin() -> &'input str = "0b" s:$(['0'..='1']+) { s }

    rule _typ() -> &'input str = $( ['u']? ['l' | 'h' | 's' | 'b' | 'c']? )
    // ===================================================

    // char ===============================================
    pub rule any_char() -> char = c:$( [_] ) { c.chars().nth(0).unwrap() }

    pub rule raw_bytes() -> Vec<u8> = ( visible_byte() / escaped_byte() )*

    pub rule visible_byte() -> u8 = !['\\'] c:$(['!'..='~']) { *c.as_bytes().first().unwrap() }
    pub rule escaped_byte() -> u8 = ['\\'] c:escaped_byte_() { c }

    rule escaped_byte_() -> u8
      = escaped_hex()
      / escaped_oct()
      / escaped_asc()

    rule escaped_hex() -> u8
      = ['x'] c:$(['0'..='9' | 'a'..='f' | 'A'..='F']*<2>) {
        u8::from_str_radix(c, 16).unwrap()
      }

    rule escaped_oct() -> u8
      = c:$(['0'..='7']*<3>) {
        u8::from_str_radix(c, 8).unwrap()
      }

    rule escaped_asc() -> u8
      = c:any_char() {
        (match c {
            't' => '\t', '0' => '\0', 'n' => '\n', 'r' => '\r',
            'v' => '\u{b}', 'a' => '\u{7}', 'b' => '\u{8}', 'f' => '\u{c}',
            unk => unk
        }) as u8
      }
    // ====================================================

    // basic ==============================================
    rule _() = [' ']
    rule __() = [' ' | '\t' | '\r']+

    rule eps() = ""
    rule peek_whitespace() = &[' ' | '\t' | '\r' | '\0'] / ![_]
    rule peek_non_whitespace() = ![' ' | '\t' | '\r' | '\0'] &[_]
    rule backspace() = ['\u{8}'] / ['\\'] ['b']

    rule ident() ->&'input str = $( !['0'..='9'] ['0'..='9' | 'a'..='z' | 'A'..='Z']+ )
    // ====================================================
  }
}
