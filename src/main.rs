use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;
mod error;

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
            Ok(games) => {
                for mut g in games {
                    match bgg.fill_details(&mut g) {
                        Err(e) => {
                            print_err(e);
                            break;
                        }
                        Ok(_) => println!("{:?}", g),
                    }
                }
            }
        },
        Commands::Detail { id } => match bgg.detail(id) {
            Err(e) => print_err(e),
            Ok(detail) => println!("{:?}", detail),
        },
    }
}
