use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;

use cli::{Cli, Commands};

fn print_err(msg: &str) {
    println!("ERR - {}", msg);
}

fn main() {
    let cli = Cli::parse();
    let bgg = Bgg::new();
    match &cli.command {
        Commands::Collection { user } => match bgg.collection(user, true) {
            Err(msg) => print_err(msg),
            Ok(games) => {
                for mut g in games {
                    match bgg.fill_details(&mut g) {
                        Err(msg) => println!("ERR - {}", msg),
                        Ok(_) => println!("{:?}", g),
                    }
                }
            }
        },
        Commands::Detail { id } => match bgg.detail(id) {
            Err(msg) => print_err(msg),
            Ok(detail) => println!("{:?}", detail),
        },
    }
}
