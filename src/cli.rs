use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
pub struct Cli {
    /// Display extra information when outputting game details.
    #[arg(short, long)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List a user's collection. Allows for selection of what to list: all
    /// games owned, mechanics of all games owned or categories of all games
    /// owned. When outputting games, it supports filtering of games that
    /// contain the search string in their name, a mechanic or a category.
    Collection(CollectionCommand),
    /// Retrieve the details of a specific game give by its ID.
    Detail(DetailCommand),
    /// Search for a game using a search string.
    Search(SearchCommand),
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

#[derive(Parser)]
pub struct CollectionCommand {
    /// BGG user name to retrieve collection for
    pub user: String,
    /// Selects what data to output or filter on
    #[arg(value_enum, short, long, default_value_t=Data::Games)]
    pub data: Data,
    /// Only output games that have matching "data"
    #[arg(short, long)]
    pub filter: Option<String>,
    /// When outputting "games" sort them by this criteria
    #[arg(value_enum, short, long)]
    pub sort: Option<SortOrder>,
}

#[derive(Parser)]
pub struct DetailCommand {
    /// BGG game ID.
    pub id: u32,
}

#[derive(Parser)]
pub struct SearchCommand {
    /// (Partial) name of the game(s) being searched for.
    pub name: String,
}
