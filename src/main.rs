mod cfg;
mod env;
mod err;
mod utils;

use std::{net::TcpListener, thread::spawn};

use cfg::Config;
use err::*;
use utils::*;

fn main() -> Result<()> {
    let cfg = Config::new();
    let server = TcpListener::bind(env::SERVER_ADDR)?;
    let mut limiter = RateLimiter::new(cfg.rate_limit());

    println!("Listening @ http://{}\n", server.local_addr()?);

    // handle each stream individually
    for s in server.incoming().filter_map(|s| limiter.check(s)) {
        spawn(move || {
            if let Err(e) = handle_stream(s) {
                eprintln!("{}", e)
            }
        });
    }
    unreachable!()
}
