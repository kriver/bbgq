use crate::{
    bgg_api::Bgg,
    cli::{CollectionCommand, DetailCommand, SearchCommand},
    error::Error,
    game::Game,
};

pub trait Command {
    fn get_games(&self, bgg: &Bgg) -> Result<Vec<Game>, Error>;
}

impl Command for CollectionCommand {
    fn get_games(&self, bgg: &Bgg) -> Result<Vec<Game>, Error> {
        bgg.collection(&self.user, true)
    }
}

impl Command for DetailCommand {
    fn get_games(&self, bgg: &Bgg) -> Result<Vec<Game>, Error> {
        bgg.detail(self.id).map(|d| vec![d])
    }
}

impl Command for SearchCommand {
    fn get_games(&self, bgg: &Bgg) -> Result<Vec<Game>, Error> {
        bgg.search(&self.name)
    }
}
