use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;

use cli::Args;

fn main() {
    let args = Args::parse();
    let bgg = Bgg::new();
    match bgg.collection(&args.user, true) {
        Err(msg) => println!("ERR - {}", msg),
        Ok(games) => {
            for mut g in games {
                bgg.fill_details(&mut g);
                println!("{:?}", g);
            }
        }
    }
}
