use std::cell::RefCell;
use std::cmp;
use std::fmt::Debug;
use std::io;
use std::iter::Peekable;
use std::rc::Rc;

use crate::expr::Operator;
use crate::expr_value::ValType;
use crate::magic_line::{AuxLine, AuxStrength, AuxType, MagicLine};
use crate::tree::{TreeNode, TreeNodeBuilder};

const STRENGTH_UNIT: i32 = 10;

/// Magic entry to provide a sequence of check rules
///
/// A magic entry, which consists a list of magic lines, starts
/// from a magic line with continue-level 0 and ends at the start
/// of another magic entry.
#[derive(Debug)]
pub(crate) struct MagicEntry {
    /// a sequence of rule lines
    pub(crate) root: Rc<RefCell<TreeNode<MagicLine>>>,
    /// represents for the priority of entry
    strength: i32,
}

impl MagicEntry {
    pub(crate) fn strength(&self) -> i32 {
        self.strength
    }
}

#[derive(Debug, Default)]
pub(crate) struct MagicEntryBuilder {
    last_node: Option<Rc<RefCell<TreeNode<MagicLine>>>>,
    root: Option<Rc<RefCell<TreeNode<MagicLine>>>>,
    lines: Vec<Rc<RefCell<TreeNode<MagicLine>>>>,
    factor: Option<AuxStrength>,
}

impl MagicEntryBuilder {
    pub(crate) fn new() -> MagicEntryBuilder {
        MagicEntryBuilder {
            last_node: None,
            root: None,
            lines: vec![],
            factor: None,
        }
    }

    pub(crate) fn parse_until_new_entry<P>(mut self, lines: &mut Peekable<P>) -> Self
        where
            P: Iterator<Item=io::Result<String>>,
    {
        while let Some(line_res) = lines.peek() {
            if let Ok(line) = line_res {
                if self.is_new_entry(line) {
                    break;
                }
                self.parse_line(line.as_str())
            }
            lines.next();
        }
        self
    }

    fn parse_line(&mut self, line: &str) {
        println!("parsing {}", line);
        let mut chars = line.chars();
        match chars.next() {
            // blank line or comment line
            None | Some('#') => {}
            // aux line, which contains either a type or a strength
            Some('!') if Some(':') == chars.next() => match AuxLine::parse_line(&line[2..]) {
                AuxLine::Type(typ) => self.attach_aux_type_to_last_line(typ),
                AuxLine::Strength(factor) => self.attach_strength_to_this_entry(factor),
            },
            // otherwise, must be a magic line
            _ => self.add_line_into_tree(MagicLine::parse_line(line)),
        }
    }

    fn is_new_entry(&self, line: &str) -> bool {
        !self.lines.is_empty() && MagicLine::is_entry_line(line)
    }

    fn attach_aux_type_to_last_line(&mut self, typ: AuxType) {
        self.last_node
            .as_ref()
            .unwrap()
            .borrow_mut()
            .val
            .attach_aux(typ)
    }

    fn attach_strength_to_this_entry(&mut self, factor: AuxStrength) {
        self.factor = Some(factor)
    }

    fn add_line_into_tree(&mut self, line: MagicLine) {
        self.last_node = if self.root.is_none() {
            // if this line is the first line, then init the root
            let new_last = TreeNodeBuilder::new(line).build_ref();
            self.root = Some(new_last.clone());
            Some(new_last)
        } else {
            // else add this line into the childs of its father, whose cont_lvl is
            // exact one less than this line's
            let last_node = self.last_node.as_ref().unwrap().clone();
            let father_cont_lvl = line.cont_lvl - 1;
            let parent = TreeNode::backspace(last_node, |x| x.cont_lvl == father_cont_lvl);
            let new_last = TreeNode::add_child(parent, line);
            Some(new_last)
        };
        self.lines.push(self.last_node.as_ref().unwrap().clone());
    }

    pub(crate) fn build(self) -> Option<MagicEntry> {
        if self.root.is_none() { None } else {
            Some(MagicEntry {
                root: self.root.clone().unwrap(),
                strength: self.compute_strength(),
            })
        }
    }

    /// Compute the priority of this entry
    ///
    /// The priority is used to sort all the entries so that the interested
    /// or the cheapper rules can be applied first.
    fn compute_strength(&self) -> i32 {
        let mut strength = STRENGTH_UNIT * 2;
        strength += self.strength_delta_from_cmp_val().unwrap_or(0);
        strength += self.strength_delta_from_cmp_op().unwrap_or(0);
        self.strength_by_factor(strength).unwrap_or(1)
    }

    /// Compute the strength contributed by the the comparison value
    fn strength_delta_from_cmp_val(&self) -> Option<i32> {
        let magic_line = &self.last_node.as_ref().unwrap().borrow().val;
        let fn_str_len = || Some(magic_line.reln_val()?.str_len()? as i32);
        Some(match magic_line.typ.typ {
            typ if typ.is_i8() => STRENGTH_UNIT,
            typ if typ.is_i16() => STRENGTH_UNIT * 2,
            typ if typ.is_i32() => STRENGTH_UNIT * 4,
            typ if typ.is_i64() => STRENGTH_UNIT * 8,
            ValType::PString | ValType::String => STRENGTH_UNIT * fn_str_len()?,
            ValType::LEString16 | ValType::BEString16 => STRENGTH_UNIT * fn_str_len()? / 2,
            ValType::Search => cmp::max(STRENGTH_UNIT, fn_str_len()?),
            ValType::Der | ValType::Regex => STRENGTH_UNIT,
            _ => 0,
        })
    }

    /// Compute the strength contributed by the the comparison operation
    fn strength_delta_from_cmp_op(&self) -> Option<i32> {
        let magic_line = &self.last_node.as_ref().unwrap().borrow().val;
        Some(match magic_line.reln_op()? {
            Operator::EQ => STRENGTH_UNIT,
            Operator::LT | Operator::GT => -STRENGTH_UNIT * 2,
            Operator::And | Operator::Xor => -STRENGTH_UNIT,
            _ => 0,
        })
    }

    /// Apply the factor provied by magic file author to the accumulated strength
    fn strength_by_factor(&self, strength: i32) -> Option<i32> {
        let AuxStrength { op, val } = self.factor.as_ref()?;
        Some(match op {
            Operator::Add => strength + val,
            Operator::Minus => strength - val,
            Operator::Multiply => strength * val,
            Operator::Divide => strength / val,
            _ => 0,
        })
    }
}
