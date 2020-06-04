use std::fs::File;
use std::io::{BufReader, Read, Seek};

use crate::magic_file::MagicFile;
use crate::magic_match::{MResult, MagicMatch};
use crate::expr::{Expression, UnOperator};
use crate::magic_entry::MagicEntry;
use crate::magic_line::MagicLine;
use std::rc::Rc;
use crate::expr_value::Value;

struct MatchRecord {
    matched_start: i64,
    matched_end: Rc<Value>,
    fetched_val: Option<Value>
}

pub(crate) struct MagicMatcher {
    magic: MagicFile,
    match_rec: Vec<MatchRecord>,
}

impl<S: Seek + Read> MagicMatch<S> for MagicMatcher {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> MResult {
        let mut ctx = MatchContext::new(0, 0, buf);

        for entry in &self.magic.entries {
            self.match_entry(&entry, &mut ctx)
        }

        Ok(())
    }
}

impl MagicMatcher {
    fn match_entry<S: Seek + Read>(&self, entry: &MagicEntry, ctx: &mut MatchContext<S>) {
        let root = entry.root.clone();
        let mut stack = vec![root];

        while let Some(current) = stack.pop() {
            ctx.last_offset = match self.match_rec.last() {
                Some(rec) => rec.matched_end.clone(),
                None => Rc::new(Value::I64(0)),
            };
            // todo: handle the result of match line
            //  match: add into match_rec, print and continue down
            //  unmatch: cut children and backward
            self.match_line(&current.borrow().val, ctx)
        }
    }

    fn match_line<S: Seek + Read>(&self, line: &MagicLine, ctx: &mut MatchContext<S>) {
        // todo:
        //  0. compute ofs;
        //  1. fetch from ofs
        //  2. compute reln
        //  3. record ofs, reln, val and so on
        let ofs = line.ofs.eval(ctx);
    }
}

struct MatchContext<'a, S: Seek + Read> {
    start: i64,
    last_offset: Rc<Value>,
    mem: &'a BufReader<S>,
}

impl<'a, S: Seek + Read> MatchContext<'a, S> {
    pub fn new(start: i64, last_offset: i64, mem: &'a mut BufReader<S>) -> Self {
        let last_offset = Value::I64(last_offset);
        let last_offset = Rc::new(last_offset);
        MatchContext { start, last_offset, mem }
    }
}

impl Expression {
    fn eval<S: Seek + Read>(&self, ctx: &mut MatchContext<S>) -> Rc<Value> {
        match self {
            Expression::UnOp(op, exp, act) => {
                let exp = exp.eval(ctx);
                let exp = match op {
                    UnOperator::Absolute => exp,
                    UnOperator::Relative => {
                        Expression::add(exp.clone(), ctx.last_offset.clone())
                    }
                    UnOperator::Indirect(_) => {
                        // todo indirect fetch and cast
                        exp
                    },
                };
                // todo action
                exp
            },
            Expression::Val(val) => val.clone(),
        }
        // Rc::new(Value::I64(0))
    }

    fn add(lhs: Rc<Value>, rhs: Rc<Value>) -> Rc<Value> {
        // todo
        Rc::new(Value::I64(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buff_file(path: &str) -> BufReader<File> {
        let file = File::open(path).unwrap();
        BufReader::new(file)
    }

    #[test]
    fn test() {
        let magic_file = MagicFile::parse("/usr/share/file/magic/pdf").unwrap();
        let mut matcher = MagicMatcher {
            magic: magic_file,
            match_rec: vec![]
        };

        let path = "/Users/ligengwang/Downloads/IDAPython-Book.pdf";
        let mut buff = buff_file(path);

        matcher.magic_match(&mut buff);
    }
}
