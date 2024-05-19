mod env;
mod err;

use err::*;
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread::spawn;
use tungstenite::{accept, Message};
use wlrs_auth::hash_password_from_b64;

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
    let hashed = hash_password_from_b64(&pass, env::SERVER_PASS_SALT).unwrap();

    // determine if correct password was provided
    if hashed != env::SERVER_PASS_HASH {
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

fn handle_stream(stream: TcpStream) -> Result<()> {
    let mut ws = accept(stream)?;
    let res = handle_request(ws.read()?).unwrap_or(u8::MAX);
    ws.send(Message::Binary(vec![res])).map_err(Into::into)
}

fn main() -> Result<()> {
    let server = TcpListener::bind(env::SERVER_ADDR)?;

    println!("Listening @ http://{}\n", server.local_addr()?);

    // handle each stream individually
    server.incoming().filter_map(Result::ok).for_each(|s| {
        spawn(move || {
            if let Err(e) = handle_stream(s) {
                eprintln!("{}", e)
            }
        });
    });
    unreachable!()
}
