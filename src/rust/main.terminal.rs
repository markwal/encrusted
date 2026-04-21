use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use clap::{Arg, Command};

mod buffer;
mod frame;
mod instruction;
mod options;
mod quetzal;
mod traits;
mod ui_terminal;
mod zmachine;
mod chgrid;
mod termbuffer;

use crate::options::Options;
use crate::ui_terminal::TerminalUI;
use crate::zmachine::Zmachine;

const VERSION: &str = env!("CARGO_PKG_VERSION");


fn main() {
    let matches = Command::new("encrusted")
        .version(VERSION)
        .about("A zmachine interpreter")
        .arg(
            Arg::new("FILE")
                .help("Sets the story file to run")
                .required(false)
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .help("sets the column width for wrapping text (default: 60)")
                .num_args(1)
        )
        .get_matches();

    let path = Path::new(
        matches
            .get_one::<String>("FILE")
            .map(String::as_str)
            .unwrap_or("assets/zork2.z3"),
    );
    let mut width = matches
        .get_one::<String>("width")
        .map(String::as_str)
        .unwrap_or("60")
        .parse::<u16>()
        .unwrap_or(1);

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

    let ui = TerminalUI::new(version, width);
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
