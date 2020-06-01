#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate num_derive;
extern crate peg;
extern crate regex;

use std::fs;

mod magic;
mod magic_entry;
mod magic_file;
mod magic_line;

// use clap::{App, Arg};

fn main() {
    for f in fs::read_dir("/usr/share/file/magic").unwrap() {
        let p = f.unwrap().path();
        println!("parsing {:?}...", p);
        magic_file::MagicFile::parse(p).unwrap();
    }
    println!("hello, world!");
}
