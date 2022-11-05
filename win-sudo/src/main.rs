fn main() {
    let mut args = std::env::args().skip(1);

    // <COMMAND> -> '<COMMAND>'
    // surround with single-quotes, in case of any spaces in <COMMAND>
    let mut cmd_args = match args.next() {
        None => {
            return;
        }
        Some(s) => format!("'{s}'"),
    };

    let mut args = args.map(|s| {
        // <ARG> -> "<ARG>" (normal escape) -> '"<ARG>"' (powershell-style escape)
        // surround with double-quotes then with single-quotes,
        // to avoid powershell split arguments by space;
        // escape `"` by `\"`, escape `'` by `''`
        let s = s.replace('"', r#"\""#).replace('\'', "''");
        format!(r#"'"{s}"'"#)
    });
    if args.len() > 0 {
        cmd_args.push(' ');
        while let Some(s) = args.next() {
            // <ARG> -> "<ARG>" (normal escape) -> '"<ARG>"' (powershell-style escape)
            // surround with double-quotes then with single-quotes,
            // to avoid powershell split arguments by space;
            // escape `"` by `\"`, escape `'` by `''`
            let s = s.replace('"', r#"\""#).replace('\'', "''");
            let s = format!(r#"'"{s}"'"#);
            cmd_args.push_str(&s);
            if args.len() > 0 {
                cmd_args.push(',');
            }
        }
    }

    let script = format!("& {{ Start-Process {cmd_args} -Verb RunAs }}");
    win_exec::ExecCommand::new("powershell")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-Command")
        .arg(&script)
        .exec(true);
}
