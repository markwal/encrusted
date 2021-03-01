extern crate base64;
extern crate clap;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate bitflags;
extern crate crossterm;
extern crate unicode_segmentation;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use clap::{App, Arg};

mod buffer;
mod frame;
mod instruction;
mod options;
mod quetzal;
mod traits;
mod ui_terminal;
mod zmachine;
mod termbuffer;

use options::Options;
use ui_terminal::TerminalUI;
use zmachine::Zmachine;

const VERSION: &str = env!("CARGO_PKG_VERSION");


fn main() {
    let matches = App::new("encrusted")
        .version(VERSION)
        .about("A zmachine interpreter")
        .arg(
            Arg::with_name("FILE")
                .help("Sets the story file to run")
                .required(false)
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .help("sets the column width for wrapping text (default: 60)")
                .takes_value(true)
        )
        .get_matches();

    let path = Path::new(matches.value_of("FILE").unwrap_or("assets/zork2.z3"));
    let mut width = matches.value_of("width").unwrap_or("60").parse::<u16>().unwrap_or(1);

    if (1..10).contains(&width) {
        println!("\nExpected a valued from 10 to 65535 for width or 0=full terminal width.");
        process::exit(1);
    }

    if !path.is_file() {
        println!(
            "\nCouldn't find game file: \n   {}\n",
            path.to_string_lossy()
        );
        process::exit(1);
    }

    let mut data = Vec::new();
    let mut file = File::open(path).expect("Error opening file");
    file.read_to_end(&mut data).expect("Error reading file");

    let version = data[0];

    if version == 0 || version > 8 {
        println!(
            "\n\
             \"{}\" has an unsupported game version: {}\n\
             Is this a valid game file?\n",
            path.to_string_lossy(),
            version
        );
        process::exit(1);
    }

    let ui = TerminalUI::new_with_width(width);
    width = ui.width;
    let height = ui.height;

    let mut opts = Options::default();
    opts.save_dir = path.parent().unwrap().to_string_lossy().into_owned();
    opts.save_name = path.file_stem().unwrap().to_string_lossy().into_owned();

    let rand32 = || rand::random();
    opts.rand_seed = [rand32(), rand32(), rand32(), rand32()];

    let mut zvm = Zmachine::new(data, ui, opts);

    zvm.terp_caps.height = height;
    zvm.terp_caps.width = width;
    zvm.terp_caps.split_screen = true;
    zvm.restart_header();

    zvm.run();
}
