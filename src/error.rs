use colored::*;
use std::fmt;
use std::io;
use std::string::{FromUtf8Error, String};
use utf8::BufReadDecoderError;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    UTF8(),
    PATH(Vec<u8>),
    MANY(Vec<Error>),
    CUSTOM(String),
    PARSEFORMAT(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "{}: {}", "IO Error".red().bold(), err),
            Error::UTF8() => write!(f, "{}", "UTF-8 Error".red().bold()),
            Error::PATH(v) => write!(
                f,
                "{}: {}",
                "Invalid Path".red().bold(),
                String::from_utf8_lossy(v)
            ),
            Error::MANY(errs) => write!(f, "{}: {:?}", "Errors".red().bold(), errs),
            Error::CUSTOM(s) => write!(f, "{}: {}", "Error".red().bold(), s),
            Error::PARSEFORMAT(s) => write!(f, "{}: {}", "Error Parsing --format".red().bold(), s),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<Vec<Error>> for Error {
    fn from(errs: Vec<Error>) -> Error {
        Error::MANY(errs)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error::CUSTOM(msg)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::PATH(err.into_bytes())
    }
}

impl<'a> From<BufReadDecoderError<'a>> for Error {
    fn from(err: BufReadDecoderError<'a>) -> Error {
        match err {
            BufReadDecoderError::InvalidByteSequence(_) => Error::UTF8(),
            BufReadDecoderError::Io(ioerr) => Error::IO(ioerr),
        }
    }
}
