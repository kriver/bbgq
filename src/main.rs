use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;

use cli::Args;

fn main() {
    let args = Args::parse();
    let mut bgg = Bgg::new().with_user(&args.user);
    bgg.collection(true);
}
