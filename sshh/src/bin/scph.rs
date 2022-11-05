use std::process::exit;
use std::str::FromStr;

use clap::Parser;

use sshh as lib;

#[derive(Parser)]
struct Arg {
    #[arg(short, long, help = "Edit the entries")]
    edit: bool,

    #[arg(short, long = "recursive", help = "Copy entire directory recursively")]
    recursive: bool,

    #[arg(help = "Local or remote paths")]
    paths: Vec<String>,
}

fn main() {
    let arg = Arg::parse();

    if arg.edit {
        lib::edit_entries();
    }

    if arg.paths.is_empty() {
        lib::print_entries();
    }

    if arg.paths.len() == 1 {
        eprintln!("error: insufficient path argument");
        exit(1);
    }

    let entries = match lib::load_entries() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            exit(1);
        }
    };

    let mut paths = Vec::with_capacity(arg.paths.len());
    for path in &arg.paths {
        let parts = path.splitn(2, ':').collect::<Vec<&str>>();
        if parts.len() < 2 {
            paths.push(parts[0].to_string());
        } else {
            let target = match usize::from_str(parts[0]) {
                Ok(i) => i,
                Err(_) => {
                    eprintln!(
                        "error: invalid path argument '{path}'; target index is not a number"
                    );
                    exit(1);
                }
            };
            if target >= entries.len() {
                eprintln!("error: invalid path argument '{path}'; invalid target index {target}");
                exit(1);
            }
            let entry = entries[target].split(' ').next().unwrap().trim();
            let path = parts[1];
            let target_uri = format!("scp://{entry}/{path}");
            paths.push(target_uri);
        }
    }

    let mut cmd = win_exec::ExecCommand::new("scp");
    if arg.recursive {
        cmd.arg("-r");
    }
    cmd.args(paths);
    cmd.exec(true);
}
