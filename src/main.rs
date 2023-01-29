use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;

use cli::Args;

fn main() {
    let args = Args::parse();
    let bgg = Bgg::new().with_user(&args.user);
    match bgg.collection(true) {
        Err(msg) => println!("ERR - {}", msg),
        Ok(games) => println!("{:#?}", games),
    }
}
