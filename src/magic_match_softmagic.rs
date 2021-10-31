use std::fs::File;
use std::io::{BufReader, Read, Seek};

use crate::expr::{Action, Expression, UnOperator};
use crate::expr_value::{Value, SignValType, ValType};
use crate::magic_entry::MagicEntry;
use crate::magic_file::MagicFile;
use crate::magic_line::MagicLine;
use crate::magic_match::{MResult, MagicMatch};
use std::rc::Rc;

struct MatchRecord {
    matched_start: i64,
    matched_end: Rc<Value>,
    fetched_val: Option<Value>,
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
            ctx.relative_offset = match self.match_rec.last() {
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
    relative_offset: Rc<Value>,
    indirect_offset: Rc<Value>,
    indirect_type: SignValType,
    mem: &'a BufReader<S>,
}

impl<'a, S: Seek + Read> MatchContext<'a, S> {
    pub fn new(start: i64, last_offset: i64, mem: &'a mut BufReader<S>) -> Self {
        let relative_offset = Rc::new(Value::I64(last_offset));
        let indirect_offset = Rc::new(Value::I64(start));
        let indirect_type = SignValType::new(true, ValType::Quad);
        MatchContext {
            start,
            relative_offset,
            indirect_offset,
            indirect_type,
            mem,
        }
    }
}

impl Expression {
    fn eval<S: Seek + Read>(&self, ctx: &mut MatchContext<S>) -> Rc<Value> {
        match self {
            Expression::UnOp(op, exp, act) => {
                let val = exp.eval(ctx);

                // very strange mechanism: the offset of the indirect action in the right
                // hand side is based on the value of the left hand
                let old_indir_offset = ctx.indirect_offset.clone();
                ctx.indirect_offset = val.clone();
                let mask_val = match act {
                    None => None,
                    Some(action) => match action {
                        Action::Num { op, val } => Some(val.eval(ctx)),
                        Action::Str { flags, range } => {
                            // TODO need implementation
                            None
                        }
                    },
                };
                ctx.indirect_offset = old_indir_offset;

                let exp = match op {
                    UnOperator::Absolute => val,
                    UnOperator::Relative => {
                        Expression::add(val.clone(), ctx.relative_offset.clone())
                    }
                    UnOperator::Indirect(typ) => {
                        if let Some(action) = act {
                            match action {
                                Action::Num { op, val } => {}
                                Action::Str { flags, range } => {}
                            }
                        };

                        // todo indirect fetch and cast
                        val
                    }
                };

                exp
            }
            Expression::Val(val) => val.clone(),
        }
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
            match_rec: vec![],
        };

        let path = "/Users/ligengwang/Downloads/IDAPython-Book.pdf";
        let mut buff = buff_file(path);

        matcher.magic_match(&mut buff);
    }
}
