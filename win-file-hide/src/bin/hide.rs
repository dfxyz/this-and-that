use std::ffi::OsString;

use clap::Parser;

use win_file_hide as lib;

#[derive(Parser)]
struct Arg {
    #[arg(short, long, help = "Also modify the system attribute")]
    system: bool,
    #[arg(required = true, help = "File(s) to hide")]
    files: Vec<OsString>,
}

fn main() {
    let arg = Arg::parse();
    for file in arg.files {
        lib::hide(file, false, arg.system);
    }
}
