pub enum Flag {
    Help,
    Version,
    Debug,
    Verbose,
    Error,
}

pub fn parse_args(args: &Vec<String>) -> Vec<Flag> {
    let mut flags = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => flags.push(Flag::Help),
            "-v" | "--version" => flags.push(Flag::Version),
            "-d" | "--debug" => flags.push(Flag::Debug),
            "-V" | "--verbose" => flags.push(Flag::Verbose),
            "-E" | "--error" => flags.push(Flag::Error),
            _ => (),
        }
    }
    flags
}
