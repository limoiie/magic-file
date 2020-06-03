#![feature(bufreader_seek_relative)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate num_derive;
extern crate peg;
extern crate regex;

use std::fs;
use crate::magic_file::MagicFile;

mod magic;
mod magic_entry;
mod magic_file;
mod magic_line;
mod magic_match;

mod magic_match_filesystem;
mod magic_match_ascii;
mod magic_match_tar;
mod magic_match_json;
mod magic_match_softmagic;

mod raw_bytes;
mod ext_buf;
mod tree;

// use clap::{App, Arg};

fn main() {
    for f in fs::read_dir("/usr/share/file/magic").unwrap() {
        let p = f.unwrap().path();
        println!("parsing {:?}...", p);
        magic_file::MagicFile::parse(p).unwrap();
    }
    // MagicFile::parse("/usr/share/file/magic/weak");
    println!("hello, world!");
}
