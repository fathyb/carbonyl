use super::{CommandLine, CommandLineProgram};

pub fn main() -> Option<CommandLine> {
    match CommandLineProgram::parse() {
        CommandLineProgram::Main(cmd) => return Some(cmd),
        CommandLineProgram::Help => {
            println!("{}", help())
        }
        CommandLineProgram::Version => {
            println!("Carbonyl {}", env!("CARGO_PKG_VERSION"))
        }
    }

    None
}

pub fn help() -> &'static str {
    "Usage: carbonyl [options] [url]

Options:
    -h, --help                 display this help message
    -d, --debug                enable debug logs
    -v, --version              output the version number
"
}
