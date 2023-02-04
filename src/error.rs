use std::num::ParseIntError;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Request(reqwest::Error),
    Xml(roxmltree::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "E(msg) - {}", msg),
            Error::Request(e) => write!(f, "E(req) - {}", e),
            Error::Xml(e) => write!(f, "E(xml) - {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Message(_) => None,
            Error::Request(ref e) => Some(e),
            Error::Xml(ref e) => Some(e),
        }
    }
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Self {
        Error::Message(msg.to_owned())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Message(msg)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Request(e)
    }
}

impl From<roxmltree::Error> for Error {
    fn from(e: roxmltree::Error) -> Self {
        Error::Xml(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::Message(format!("{}", e))
    }
}
