use std::{env, ffi::OsStr};

use super::CommandLineProgram;

#[derive(Clone, Debug)]
pub struct CommandLine {
    pub args: Vec<String>,
    pub fps: f32,
    pub zoom: f32,
    pub debug: bool,
    pub bitmap: bool,
    pub sixel_only: bool,
    pub program: CommandLineProgram,
    pub shell_mode: bool,
}

pub enum EnvVar {
    Debug,
    Bitmap,
    SixelOnly,
    ShellMode,
}

impl EnvVar {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnvVar::Debug => "CARBONYL_ENV_DEBUG",
            EnvVar::Bitmap => "CARBONYL_ENV_BITMAP",
            EnvVar::SixelOnly => "CARBONYL_ENV_SIXEL_ONLY",
            EnvVar::ShellMode => "CARBONYL_ENV_SHELL_MODE",
        }
    }
}

impl AsRef<OsStr> for EnvVar {
    fn as_ref(&self) -> &OsStr {
        self.as_str().as_ref()
    }
}

impl CommandLine {
    pub fn parse() -> CommandLine {
        let mut fps = 60.0;
        let mut zoom = 1.0;
        let mut debug = false;
        let mut bitmap = false;
        let mut sixel_only = true;
        let mut shell_mode = false;
        let mut program = CommandLineProgram::Main;
        let args = env::args().skip(1).collect::<Vec<String>>();

        for arg in &args {
            let split: Vec<&str> = arg.split("=").collect();
            let default = arg.as_str();
            let (key, value) = (split.get(0).unwrap_or(&default), split.get(1));

            macro_rules! set {
                ($var:ident, $enum:ident) => {{
                    $var = true;

                    env::set_var(EnvVar::$enum, "1");
                }};
            }

            macro_rules! set_f32 {
                ($var:ident = $expr:expr) => {{
                    if let Some(value) = value {
                        if let Some(value) = value.parse::<f32>().ok() {
                            $var = {
                                let $var = value;

                                $expr
                            };
                        }
                    }
                }};
            }

            match *key {
                "-f" | "--fps" => set_f32!(fps = fps),
                "-z" | "--zoom" => set_f32!(zoom = zoom / 100.0),
                "-d" | "--debug" => set!(debug, Debug),
                "-b" | "--bitmap" => set!(bitmap, Bitmap),
                "--sixel-only" => set!(sixel_only, SixelOnly),
                "--legacy-text" => {
                    sixel_only = false;

                    env::set_var(EnvVar::SixelOnly, "0");
                }

                "-h" | "--help" => program = CommandLineProgram::Help,
                "-v" | "--version" => program = CommandLineProgram::Version,
                _ => (),
            }
        }

        if env::var(EnvVar::Debug).is_ok() {
            debug = true;
        }

        if env::var(EnvVar::Bitmap).is_ok() {
            bitmap = true;
        }

        if let Ok(value) = env::var(EnvVar::SixelOnly) {
            let normalized = value.trim().to_ascii_lowercase();

            sixel_only = !matches!(normalized.as_str(), "0" | "false" | "off" | "no");
        }

        env::set_var(EnvVar::SixelOnly, if sixel_only { "1" } else { "0" });

        if env::var(EnvVar::ShellMode).is_ok() {
            shell_mode = true;
        }

        CommandLine {
            args,
            fps,
            zoom,
            debug,
            bitmap,
            sixel_only,
            program,
            shell_mode,
        }
    }
}
