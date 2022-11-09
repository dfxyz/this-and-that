use std::env;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::path::PathBuf;
use std::process::exit;

const FILENAME: &str = ".sshh_entries";

#[cfg(windows)]
fn entry_file_path() -> PathBuf {
    PathBuf::from(env::var("USERPROFILE").unwrap()).join(FILENAME)
}
#[cfg(not(windows))]
fn entry_file_path() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap()).join(FILENAME)
}

pub fn edit_entries() -> ! {
    let path = entry_file_path();
    let editor = match env::var("EDITOR") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("error: environment variable 'EDITOR' not defined");
            exit(1);
        }
    };
    let mut cmd = win_exec::ExecCommand::new(&editor);
    cmd.arg(&path);
    cmd.exec(true);
}

pub fn load_entries() -> Result<Vec<String>, String> {
    let file = match File::open(entry_file_path()) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Ok(vec![]);
        }
        Err(e) => {
            return Err(format!("failed to open entry file; {e}"));
        }
    };
    let mut lines = vec![];
    let mut reader = BufReader::new(file);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(len) => {
                if len == 0 {
                    break;
                }
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let line = line.to_string();
                lines.push(line);
            }
            Err(e) => {
                return Err(format!("failed to read line from entry file; {e}"));
            }
        }
    }
    Ok(lines)
}

pub fn print_entries() -> ! {
    let entries = match load_entries() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: {e}");
            exit(1);
        }
    };
    let mut writer = BufWriter::new(stdout());
    for (i, entry) in entries.iter().enumerate() {
        writeln!(&mut writer, "[{i}] {entry}").unwrap();
    }
    writer.flush().unwrap();
    exit(0);
}
