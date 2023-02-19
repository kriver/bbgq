use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List a user's collection. Allows for selection of what to list: all
    /// games owned, mechanics of all games owned or categories of all games
    /// owned. When outputting games, it supports filtering of games that
    /// contain the search string in their name, a mechanic or a category.
    Collection {
        /// BGG user name to retrieve collection for
        user: String,
        /// Selects what data to output or filter on
        #[arg(value_enum, short, long, default_value_t=Data::Games)]
        data: Data,
        /// Display extra information when outputting game details.
        #[arg(short, long)]
        verbose: bool,
        /// Only output games that have matching "data"
        #[arg(short, long)]
        filter: Option<String>,
        /// When outputting "games" sort them by this criteria
        #[arg(value_enum, short, long)]
        sort: Option<SortOrder>,
    },
    /// Retrieve the details of a specific game give by its ID.
    Detail {
        /// BGG game ID.
        id: u32,
    },
    /// Search for a game using a search string.
    Search {
        /// (Partial) name of the game(s) being searched for.
        name: String,
        /// Display extra information when outputting game details.
        #[arg(short, long)]
        verbose: bool,
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
