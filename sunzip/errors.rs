use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Error {
    IOError(String),
    SevenzError(String),
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Error", 2)?;
        s.serialize_field("variant", &self.variant())?;
        s.serialize_field("message", &format!("{}", self))?;
        s.end()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.variant(),
            match self {
                Self::IOError(s) => format!("{}", s),
                Self::SevenzError(s) => format!("{}", s),
            }
        )
    }
}

impl Error {
    pub fn variant(&self) -> String {
        match self {
            Error::IOError(_) => "IOError",
            Error::SevenzError(_) => "SevenzError",
        }
        .to_string()
    }
}

impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(format!("{}", e))
    }
}
impl From<iocore::Exception> for Error {
    fn from(e: iocore::Exception) -> Self {
        Error::IOError(format!("{}", e))
    }
}
impl From<sevenz_rust::Error> for Error {
    fn from(e: sevenz_rust::Error) -> Self {
        Error::SevenzError(format!("{}", e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
