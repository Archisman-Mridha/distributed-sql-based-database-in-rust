use std::{fmt::Display, num::{ParseFloatError, ParseIntError}};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
  Parse(String),
  Value(String)
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    unimplemented!( )
  }
}

impl std::error::Error for Error { }

impl From<ParseIntError> for Error {
  fn from(err: ParseIntError) -> Self {
    Error::Parse(err.to_string( ))
  }
}

impl From<ParseFloatError> for Error {
  fn from(err: ParseFloatError) -> Self {
    Error::Parse(err.to_string( ))
  }
}