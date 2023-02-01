use std::path::Path;

use chrono::prelude::*;

use crate::utils::try_block;

macro_rules! debug {
    ($($args:expr),+) => {
        crate::utils::log::write(
            "DEBUG",
            file!(),
            line!(),
            &format!($($args),*)
        )
    };
}
macro_rules! warning {
    ($($args:expr),+) => {
        crate::utils::log::write(
            "WARNING",
            file!(),
            line!(),
            &format!($($args),*)
        )
    };
}
macro_rules! error {
    ($($args:expr),+) => {
        crate::utils::log::write(
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
        try_block!(Path::new(file).file_name()?.to_str()).unwrap_or("default"),
        line,
        message
    );
}
