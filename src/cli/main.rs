use super::{CommandLine, CommandLineProgram};

pub fn main() -> Option<CommandLine> {
    match CommandLineProgram::parse() {
        CommandLineProgram::Main(cmd) => return Some(cmd),
        CommandLineProgram::Help => {
            println!("{}", include_str!("usage.txt"))
        }
        CommandLineProgram::Version => {
            println!("Carbonyl {}", env!("CARGO_PKG_VERSION"))
        }
    }

    None
}
