use std::net::TcpStream;

use tungstenite::{handshake::server::NoCallback, ServerHandshake};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum ErrorKind {
    MissingInitialLength,
    UnexpectedElement,
}

impl ToString for ErrorKind {
    fn to_string(&self) -> String {
        match self {
            Self::MissingInitialLength => "Missing initial length specifier",
            Self::UnexpectedElement => "Unexpected element found",
        }
        .to_string()
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self(value.to_string())
    }
}

pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self(value.to_string())
    }
}

impl From<tungstenite::Error> for Error {
    fn from(value: tungstenite::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<tungstenite::HandshakeError<ServerHandshake<TcpStream, NoCallback>>> for Error {
    fn from(value: tungstenite::HandshakeError<ServerHandshake<TcpStream, NoCallback>>) -> Self {
        Self(value.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}
