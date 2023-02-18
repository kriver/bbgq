use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Collection {
        /// BGG user name to retrieve collection for
        user: String,
        /// Selects what data to output
        #[arg(value_enum, short, long, default_value_t=Data::Games)]
        data: Data,
        /// When outputting games sort them by this criteria
        #[arg(value_enum, short, long)]
        sort: Option<SortOrder>,
    },
    Detail {
        id: u32,
    },
    Search {
        name: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum SortOrder {
    Name,
    Rating,
    Rank,
}

#[derive(Clone, ValueEnum)]
pub enum Data {
    Games,
    Mechanics,
    Categories,
}
