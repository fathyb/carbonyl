use std::path::Path;

use chrono::prelude::*;

macro_rules! debug {
    ($($args:expr),+) => {
        crate::log::write(
            "DEBUG",
            file!(),
            line!(),
            &format!($($args),*)
        )
    };
}
macro_rules! warning {
    ($($args:expr),+) => {
        crate::log::write(
            "WARNING",
            file!(),
            line!(),
            &format!($($args),*)
        )
    };
}
macro_rules! error {
    ($($args:expr),+) => {
        crate::log::write(
            "ERROR",
            file!(),
            line!(),
            &format!($($args),*)
        )
    };
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use warning;

pub fn write(level: &str, file: &str, line: u32, message: &str) {
    let date = Utc::now();

    eprintln!(
        "[{:02}{:02}/{:02}{:02}{:02}.{:06}:{}:{}({})] {}",
        date.month(),
        date.day(),
        date.hour(),
        date.minute(),
        date.second(),
        date.nanosecond() / 1000,
        level,
        file_name(file).unwrap_or("<unknown>"),
        line,
        message
    );
}

fn file_name(path: &str) -> Option<&str> {
    Path::new(path).file_name()?.to_str()
}
