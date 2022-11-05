use std::fs::{File, OpenOptions};
use std::io::{sink, stdin};
use std::ops::Sub;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

use aes_gcm::AeadInPlace;
use clap::Parser;
use rand::{CryptoRng, RngCore};

use file_crypto as lib;

#[derive(Parser)]
struct Arg {
    #[arg(short, long, help = "The secret key")]
    key: String,

    #[arg(short, long, help = "The extension name of encrypted file")]
    #[arg(default_value = "enc")]
    ext_name: String,

    #[arg(short = 'n', long, help = "Dry run without writing the result")]
    dry_run: bool,

    #[arg(
        short,
        long = "force",
        help = "Overwrite existed file without confirmation"
    )]
    overwrite: bool,

    #[arg(long = "256", help = "Use AES-256-GCM instead of AES-128-GCM")]
    use_aes256gcm: bool,

    #[arg(required = true, help = "File(s) to encrypt")]
    files: Vec<String>,
}

fn main() {
    let arg = match parse_and_check_arg() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("error: {msg}");
            exit(1);
        }
    };

    let mut rng = rand::thread_rng();
    if arg.use_aes256gcm {
        let cipher = lib::new_aes256gcm_cipher(&arg.key);
        for file in &arg.files {
            encrypt_file(&arg, &cipher, &mut rng, file);
        }
    } else {
        let cipher = lib::new_aes128gcm_cipher(&arg.key);
        for file in &arg.files {
            encrypt_file(&arg, &cipher, &mut rng, file);
        }
    }
}

fn parse_and_check_arg() -> Result<Arg, String> {
    let arg = Arg::parse();

    if arg.key.is_empty() {
        return Err("empty secret key".to_string());
    }

    if arg.ext_name.is_empty() {
        return Err("empty extension name".to_string());
    }

    for file in &arg.files {
        let path = PathBuf::from(file);
        if !path.exists() {
            return Err(format!("'{file}' not exists"));
        }
        if !path.is_file() {
            return Err(format!("'{file}' is not a file"));
        }
    }

    Ok(arg)
}

fn encrypt_file<C: AeadInPlace, RNG: CryptoRng + RngCore>(
    arg: &Arg,
    cipher: &C,
    rng: &mut RNG,
    file: &str,
) {
    let in_path = PathBuf::from(file);
    let mut in_file = match File::open(&in_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: cannot open input file '{file}'; {e}");
            return;
        }
    };
    let mut out_file = if arg.dry_run {
        None
    } else {
        let out_path = match in_path.file_name() {
            None => {
                eprintln!("error: cannot determine the file name of '{file}'");
                return;
            }
            Some(in_name) => {
                let in_name = in_name.to_string_lossy();
                let ext_name = &arg.ext_name;
                PathBuf::from(format!("{in_name}.{ext_name}"))
            }
        };
        if out_path.exists() {
            if !out_path.is_file() {
                eprintln!(
                    "error: output file '{}' exists and is not a file",
                    out_path.to_string_lossy()
                );
                return;
            }
            if !arg.overwrite {
                print!(
                    "question: overwrite file '{}'? [y/n] ",
                    out_path.to_string_lossy()
                );
                let mut input = String::new();
                match stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("error: failed to read from stdin; {e}");
                        return;
                    }
                }
                let input = input.trim();
                if input != "y" && input != "Y" {
                    return;
                }
            }
        }
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&out_path)
        {
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!(
                    "error: cannot open output file '{}': {e}",
                    out_path.to_string_lossy()
                );
                return;
            }
        }
    };

    let t0 = Instant::now();
    let result = match out_file.as_mut() {
        None => {
            let mut sink = sink();
            lib::encrypt(&mut in_file, &mut sink, cipher, rng)
        }
        Some(out_file) => {
            lib::encrypt(&mut in_file, out_file, cipher, rng)
        }
    };
    match result {
        Ok(_) => {
            let duration = Instant::now().sub(t0).as_secs_f32();
            println!("info: '{file}' encrypted; duration={duration:.3}s");
        }
        Err(msgs) => {
            eprintln!("error: failed to encrypt '{file}'; {msgs:?}");
        }
    }
}
