use std::path::PathBuf;

const ENVS: [(&str, &str); 4] = [
    ("CHERE_INVOKING", "1"),
    ("MSYSTEM", "MINGW64"),
    ("MSYS", "winsymlinks:nativestrict"),
    ("MSYS2_PATH_TYPE", "inherit"),
];

const SHELLS: [&str; 2] = ["zsh.exe", "bash.exe"];

fn main() {
    let args = std::env::args().skip(1);
    let mut shell = None;
    for sh in SHELLS {
        shell = which(sh);
        if shell.is_some() {
            break;
        }
    }
    match shell {
        None => {
            eprintln!("error: cannot find any shell");
            std::process::exit(1);
        }
        Some(shell) => {
            win_exec::ExecCommand::new(&shell)
                .args(args)
                .envs(ENVS)
                .exec(true);
        }
    }
}

fn which(cmd: &str) -> Option<PathBuf> {
    let path = match std::env::var("PATH") {
        Ok(s) => s,
        Err(_) => return None,
    };
    for dir in path.split(';') {
        let path = PathBuf::from(dir).join(cmd);
        if path.exists() {
            return Some(path);
        }
    }
    None
}
