#[macro_use]
extern crate bitflags;
extern crate regex;
extern crate num_derive;
#[macro_use]
extern crate lazy_static;
extern crate peg;


mod str_utils;
// mod magic_bk;
// mod magic_param;
// mod parse_magic_line;
// mod parse_magic_aux_line;
mod parse_magic_entry;
mod magic;
mod magic_line;
mod magic_line_parse;

use std::path::Path;
use std::fs;

// use clap::{App, Arg};

fn main() {
    for f in fs::read_dir("/usr/share/file/magic").unwrap() {
        let p = f.unwrap().path();
        println!("parsing {:?}...", p);
        parse_magic_entry::MagicFile::parse(p);
    }
    println!("hello, world!");
}
