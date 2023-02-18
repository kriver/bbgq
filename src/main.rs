use std::{cmp::Ordering, collections::HashSet};

use bgg_api::Bgg;
use clap::Parser;

mod bgg_api;
mod cli;
mod constants;
mod details;
mod error;
mod game;
mod xml_util;

use cli::*;
use details::Details;
use error::Error;
use game::Game;

fn print_err(e: Error) {
    println!("{}", e);
}

// partial because of f32
fn ord_for_option<T: PartialOrd>(a: Option<T>, b: Option<T>, reverse: bool) -> Ordering {
    match a {
        None => match b {
            None => Ordering::Equal,
            Some(_) => Ordering::Greater, // None at the end
        },
        Some(va) => match b {
            None => Ordering::Less, // None at the end
            Some(vb) => {
                let ord = va.partial_cmp(&vb).unwrap();
                if reverse {
                    ord.reverse()
                } else {
                    ord
                }
            }
        },
    }
}

fn comparator(order: &SortOrder) -> impl FnMut(&Game, &Game) -> Ordering {
    match order {
        SortOrder::Name => |a: &Game, b: &Game| a.name.cmp(&b.name),
        SortOrder::Rating => |a: &Game, b: &Game| {
            ord_for_option(
                a.details.as_ref().map(|d| d.rating),
                b.details.as_ref().map(|d| d.rating),
                true,
            )
        },
        SortOrder::Rank => |a: &Game, b: &Game| {
            ord_for_option(
                a.details.as_ref().map(|d| d.rank).flatten(),
                b.details.as_ref().map(|d| d.rank).flatten(),
                false,
            )
        },
    }
}

fn list_games(mut games: Vec<Game>, sort_by: &Option<SortOrder>) {
    if let Some(by) = sort_by {
        games.sort_by(comparator(by))
    }
    for g in games {
        println!("{}", g)
    }
}

fn list_properties(games: Vec<Game>, getter: fn(Details) -> Vec<String>) {
    let mut mechanics: Vec<String> = games
        .into_iter()
        .map(|g| g.details)
        .filter(|d| d.is_some())
        .map(|d| d.unwrap())
        .flat_map(getter)
        .collect::<HashSet<String>>() // remove duplicates
        .into_iter()
        .collect();
    mechanics.sort();
    for m in mechanics {
        println!("{}", m);
    }
}

fn list_collection(games: Vec<Game>, data: &Data, sort_by: &Option<SortOrder>) {
    match data {
        Data::Games => list_games(games, sort_by),
        Data::Mechanics => list_properties(games, |d| d.mechanics),
        Data::Categories => list_properties(games, |d| d.categories),
    }
}

fn main() {
    let cli = Cli::parse();
    let bgg = Bgg::new();
    match &cli.command {
        Commands::Collection { user, data, sort } => match bgg.collection(user, true) {
            Err(e) => print_err(e),
            Ok(mut games) => match bgg.fill_details(&mut games) {
                Err(e) => print_err(e),
                Ok(_) => list_collection(games, data, sort),
            },
        },
        Commands::Detail { id } => match bgg.detail(*id) {
            Err(e) => print_err(e),
            Ok(game) => println!("{}", game),
        },
        Commands::Search { name } => match bgg.search(name) {
            Err(e) => print_err(e),
            Ok(results) => list_games(results, &None),
        },
    }
}
