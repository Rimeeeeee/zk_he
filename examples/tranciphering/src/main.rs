mod client;
mod server;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <server|client>");
        return;
    }

    match args[1].as_str() {
        "server" => server::run().unwrap(),
        "client" => client::run().unwrap(),
        _ => println!("Unknown mode, use 'server' or 'client'"),
    }
}
