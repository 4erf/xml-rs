use std::io::{self, BufRead};

pub use self::error::{Result, Error};

mod lexer;
mod config;
mod error;
#[cfg(feature = "encodings")]
mod encodings;
