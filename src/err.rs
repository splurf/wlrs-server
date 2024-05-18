use std::net::TcpStream;
use tungstenite::{handshake::server::NoCallback, ServerHandshake};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum Error {
    IO(std::io::Error),
    Parse(std::string::FromUtf8Error),
    WebSocket(tungstenite::Error),
    Handshake(tungstenite::HandshakeError<ServerHandshake<TcpStream, NoCallback>>),
    Misc(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Parse(value)
    }
}

impl From<tungstenite::Error> for Error {
    fn from(value: tungstenite::Error) -> Self {
        Self::WebSocket(value)
    }
}

impl From<tungstenite::HandshakeError<ServerHandshake<TcpStream, NoCallback>>> for Error {
    fn from(value: tungstenite::HandshakeError<ServerHandshake<TcpStream, NoCallback>>) -> Self {
        Self::Handshake(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &str) -> Self {
        Self::Misc(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Misc(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::IO(e) => e.to_string(),
            Self::Parse(e) => e.to_string(),
            Self::WebSocket(e) => e.to_string(),
            Self::Handshake(e) => e.to_string(),
            Self::Misc(e) => e.to_string(),
        })
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}
