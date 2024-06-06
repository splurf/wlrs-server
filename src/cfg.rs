use clap::Parser;

#[derive(Debug, Parser)]
pub struct Config {
    #[arg(short, long, default_value_t = 25)]
    rate_limit: u8,
}

impl Config {
    pub fn new() -> Self {
        Self::parse()
    }

    pub const fn rate_limit(&self) -> u8 {
        self.rate_limit
    }
}
