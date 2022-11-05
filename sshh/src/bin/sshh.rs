use std::process::exit;

use clap::Parser;

use sshh as lib;

#[derive(Parser)]
struct Arg {
    #[arg(short, long, help = "Edit the entries")]
    edit: bool,

    #[arg(help = "The index of remote target")]
    target: Option<usize>,

    #[arg(help = "Command to be executed in remote target")]
    command: Option<String>,

    #[arg(
        help = "The arguments of the command",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    args: Vec<String>,
}
fn main() {
    let arg = Arg::parse();

    if arg.edit {
        lib::edit_entries();
    }

    let target = match arg.target {
        None => {
            lib::print_entries();
        }
        Some(i) => i,
    };
    let entries = match lib::load_entries() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            exit(1);
        }
    };
    if target >= entries.len() {
        eprintln!("error: invalid target index {target}");
        exit(1);
    }
    let entry = entries[target].split(' ').next().unwrap().trim();
    let target_uri = format!("ssh://{entry}");
    let mut cmd = win_exec::ExecCommand::new("ssh");
    cmd.arg(target_uri);
    match arg.command {
        None => {}
        Some(s) => {
            cmd.arg(s);
        }
    }
    cmd.args(arg.args);
    cmd.exec(true);
}
