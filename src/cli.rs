use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Collection {
        user: String,
        #[arg(value_enum, short, long)]
        sort: Option<SortOrder>,
    },
    Detail {
        id: u32,
    },
}

#[derive(Clone, ValueEnum)]
pub enum SortOrder {
    Name,
    Rating,
    Rank,
}
