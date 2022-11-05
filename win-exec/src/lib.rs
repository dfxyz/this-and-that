use std::ffi::OsStr;

pub struct ExecCommand {
    cmd: std::process::Command,
}
impl ExecCommand {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let cmd = std::process::Command::new(program);
        Self { cmd }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.cmd.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.cmd.envs(vars);
        self
    }

    pub fn exec(&mut self, ignore_ctrl_c: bool) -> ! {
        if ignore_ctrl_c {
            use winapi::shared::minwindef::TRUE;
            use winapi::um::consoleapi::SetConsoleCtrlHandler;
            unsafe { SetConsoleCtrlHandler(None, TRUE) };
        }
        match self.cmd.spawn() {
            Ok(mut ch) => match ch.wait() {
                Ok(es) => match es.code() {
                    None => {
                        eprintln!("error: sub-process ended without code");
                        std::process::exit(1);
                    }
                    Some(code) => {
                        std::process::exit(code);
                    }
                },
                Err(e) => {
                    eprintln!("error: failed to wait sub-process; {e}");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("error: failed to spawn sub-process; {e}");
                std::process::exit(1);
            }
        }
    }
}
