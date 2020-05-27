#[macro_use]
extern crate bitflags;
extern crate regex;
extern crate num_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod str_utils;
mod magic;
mod magic_param;
mod parse_magic_line;
mod parse_magic_aux_line;
mod parse_magic_entry;

use std::path::Path;

// use clap::{App, Arg};

fn load_one_magic(_magic_file: &Path) {
//    init magic_set (magic_open -> file_ms_alloc)
//    load magic_set (
//      load ->
//        magic_load ->
//          file_apprentice ->
//            (for each magic-filepath)
//               apprentice_1 ->
//                 map = apprentice_load ->
//                   (for each magic-file)
//                     load_1
//                     sort by apprentice_sort
//                     set_text_binary
//                     set_last_default
//                     coalesce_entries
//                 add_mlist(ms->mlist, map) )
}


fn main() {
    // let path = Path::new("/usr/share/file/magic/acorn");
    // load_one_magic(path).expect("Failed to load one magic file!");
    println!("hello, world!");
}
