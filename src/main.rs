use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;
mod constants;
mod details;
mod error;
mod game;
mod xml_util;

use cli::{Cli, Commands};
use error::Error;

fn print_err(e: Error) {
    println!("{}", e);
}

fn main() {
    let cli = Cli::parse();
    let bgg = Bgg::new();
    match &cli.command {
        Commands::Collection { user } => match bgg.collection(user, true) {
            Err(e) => print_err(e),
            Ok(mut games) => match bgg.fill_details(&mut games) {
                Err(e) => print_err(e),
                Ok(_) => {
                    for g in games {
                        println!("{:?}", g)
                    }
                }
            },
        },
        Commands::Detail { id } => match bgg.detail(*id) {
            Err(e) => print_err(e),
            Ok(detail) => println!("{:?}", detail),
        },
    }
}
