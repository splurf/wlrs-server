use std::{
    collections::HashMap,
    net::{IpAddr, TcpStream},
    process::Command,
};
use tungstenite::{accept, Message};
use wlrs_auth::hash_password_from_b64;

use crate::{env, err::*};

pub struct RateLimiter {
    inner: HashMap<IpAddr, u8>,
    limit: u8,
}

impl RateLimiter {
    pub fn new(limit: u8) -> Self {
        Self {
            inner: Default::default(),
            limit,
        }
    }

    pub fn check(&mut self, s: Result<TcpStream, std::io::Error>) -> Option<TcpStream> {
        let s = s.ok()?;
        let ip = s.local_addr().ok()?.ip();
        (*self.inner.entry(ip).and_modify(|e| *e += 1).or_insert(1) <= self.limit).then_some(s)
    }
}

fn parse_request(msg: Message) -> Result<(String, Vec<u8>)> {
    let mut iter = msg.into_data().into_iter();

    // length of username
    let n = iter
        .next()
        .ok_or(Error::from(ErrorKind::MissingInitialLength))? as usize;

    // take `n` numebr of bytes to retrieve the username
    let mut user = String::with_capacity(n);
    for _ in 0..n {
        user.push(
            iter.next()
                .ok_or(Error::from(ErrorKind::UnexpectedElement))? as char,
        )
    }
    // the remaining bytes make up the password
    Ok((user, iter.collect()))
}

fn handle_request(msg: Message) -> Result<u8> {
    // parse username and password from client message
    let (user, pass) = parse_request(msg)?;

    // hash the provided password
    let hashed = hash_password_from_b64(&pass, wlrs::SERVER_PASS_SALT).unwrap();

    // determine if correct password was provided
    if hashed != wlrs::SERVER_PASS_HASH {
        return Ok(4); // IncorrectPassword
    }

    // interface `mcrcon` tool
    let stdout = Command::new("mcrcon")
        .args([
            "-p",
            env::RCON_PASS,
            format!("whitelist add {}", user).as_str(),
        ])
        .output()?
        .stdout;

    let status = if stdout.is_empty() {
        0 // ServerDown
    } else {
        let res = String::from_utf8(stdout)?;

        if res.starts_with("That player does not exist")
            || res.starts_with("Incorrect argument for command")
        {
            1 // PlayerNotFound
        } else if res.starts_with("Player is already whitelisted") {
            2 // Whitelisted
        } else if res.starts_with("Added") {
            3 // Success
        } else {
            u8::MAX // Unexpected
        }
    };
    Ok(status)
}

pub fn handle_stream(stream: TcpStream) -> Result<()> {
    let mut ws = accept(stream)?;
    let res = handle_request(ws.read()?).unwrap_or(u8::MAX);
    ws.send(Message::Binary(vec![res])).map_err(Into::into)
}
