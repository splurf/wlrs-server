mod env;
mod err;

use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread::spawn;
use tungstenite::{accept, Message};

fn parse_request(data: Vec<u8>) -> err::Result<(String, Vec<u8>)> {
    let mut iter = data.into_iter();
    let init = iter
        .next()
        .ok_or(err::Error::from("Missing initial length"))? as usize;

    let mut user = Vec::with_capacity(init);
    for _ in 0..init {
        user.push(
            iter.next()
                .ok_or(err::Error::from("Found unexpected element"))?,
        )
    }
    Ok((String::from_utf8(user)?, iter.collect()))
}

fn handle_interaction(msg: Message) -> err::Result<usize> {
    // user provided password
    let (user, pass) = parse_request(msg.into_data())?;

    // hash provided password
    let hash = wlrs_auth::hash_password_from_salt(&pass, wlrs::WLRS_SERVER_PASS_SALT)?;

    // compare hashes
    if hash != wlrs::WLRS_SERVER_PASS_HASH {
        return Ok(4); // IncorrectPassword
    }

    // interface `mcrcon` tool
    let stdout = Command::new("mcrcon")
        .args([
            "-p",
            env::WLRS_RCON_PASS,
            &format!("whitelist add {}", user),
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
            255 // Unexpected
        }
    };
    Ok(status)
}

fn handle_stream(stream: TcpStream) -> err::Result<()> {
    let mut ws = accept(stream)?;
    let result = handle_interaction(ws.read()?).unwrap_or(255) as u8;
    println!("{}", result);
    ws.send(Message::Binary(vec![result])).map_err(Into::into)
}

fn main() -> err::Result<()> {
    let server = TcpListener::bind(env::WLRS_SERVER_ADDR)?;

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
