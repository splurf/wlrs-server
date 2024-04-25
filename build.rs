use dotenv::{dotenv, var};
use std::fs::File;
use std::io::Write;

fn pub_const_fmt(key: &str) -> String {
    let value = var(key).unwrap();

    format!(
        "pub const {}: &str = \"{}\";\n",
        key,
        value.replace('"', "\\\"")
    )
}

fn main() {
    println!("cargo:rerun-if-changed=../.env");
    let mut f = File::create("src/env.rs").unwrap();

    // use the dotenv crate to get the .env values
    dotenv().ok();
    f.write_all(b"// This file is automatically generated by build.rs\n\n")
        .unwrap();

    f.write_all(pub_const_fmt("WLRS_SERVER_ADDR").as_bytes())
        .unwrap();
    f.write_all(pub_const_fmt("RCON_PASS").as_bytes()).unwrap();
}