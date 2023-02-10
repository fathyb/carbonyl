use std::env;

pub struct CommandLine {
    pub args: Vec<String>,
    pub debug: bool,
}

pub enum CommandLineProgram {
    Help,
    Version,
    Main(CommandLine),
}

impl CommandLineProgram {
    pub fn parse() -> CommandLineProgram {
        let mut debug = false;
        let mut args = Vec::new();

        for arg in env::args().skip(1) {
            match arg.as_str() {
                "-d" | "--debug" => debug = true,
                "-h" | "--help" => return CommandLineProgram::Help,
                "-v" | "--version" => return CommandLineProgram::Version,
                _ => args.push(arg),
            }
        }

        CommandLineProgram::Main(CommandLine { args, debug })
    }
}
