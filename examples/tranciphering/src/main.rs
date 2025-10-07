mod client;
mod server;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <server|client>");
        return;
    }

    match args[1].trim_start_matches('-') {
        "server" => server::run(),
        "client" => client::run(),
        _ => {
            println!("Unknown mode, use 'server' or 'client'");
            Ok(())
        }
    }
    .unwrap();
}
