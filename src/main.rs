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

type PropertyGetter = fn(Details) -> Vec<String>;
type PropertyGetterRef = fn(&Details) -> &[String];
type FilterType = Box<dyn FnMut(&Game) -> bool>;

fn print_err(e: Error) {
    eprintln!("{}", e);
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

fn str_contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn filter_for_property(getter: PropertyGetterRef, value: String) -> FilterType {
    Box::new(move |g: &Game| match &g.details {
        None => false,
        Some(d) => getter(d).iter().any(|s| str_contains(s, &value)),
    })
}

fn filter_for(data: Data, value: String) -> FilterType {
    match data {
        Data::Games => Box::new(move |g: &Game| str_contains(&g.name, &value)),
        Data::Mechanics => filter_for_property(|d| &d.mechanics, value),
        Data::Categories => filter_for_property(|d| &d.categories, value),
    }
}

fn print_game(g: Game, verbose: bool) {
    println!("{}", g);
    if verbose {
        if let Some(d) = g.details {
            println!("  Mechanics  : {}", d.mechanics.join(", "));
            println!("  Categories : {}", d.categories.join(", "));
        }
    }
}

fn print_games<F>(mut games: Vec<Game>, sort_by: &Option<SortOrder>, predicate: F, verbose: bool)
where
    F: FnMut(&Game) -> bool,
{
    if let Some(by) = sort_by {
        games.sort_by(comparator(by))
    }
    for g in games.into_iter().filter(predicate) {
        print_game(g, verbose);
    }
}

fn print_properties(games: Vec<Game>, getter: PropertyGetter) {
    let mut properties: Vec<String> = games
        .into_iter()
        .map(|g| g.details)
        .filter(|d| d.is_some())
        .map(|d| d.unwrap())
        .flat_map(getter)
        .collect::<HashSet<String>>() // remove duplicates
        .into_iter()
        .collect();
    properties.sort();
    for m in properties {
        println!("{}", m);
    }
}

fn print_list(
    bgg: &Bgg,
    games: Result<Vec<Game>, Error>,
    data: &Data,
    verbose: bool,
    filter: &Option<String>,
    sort_by: &Option<SortOrder>,
) {
    match games.and_then(|g| bgg.fill_details(g)) {
        Err(e) => print_err(e),
        Ok(g) => match filter {
            Some(f) => print_games(g, sort_by, filter_for(data.clone(), f.clone()), verbose),
            None => match data {
                Data::Games => print_games(g, sort_by, |_| true, verbose),
                Data::Mechanics => print_properties(g, |d| d.mechanics),
                Data::Categories => print_properties(g, |d| d.categories),
            },
        },
    }
}

fn main() {
    let cli = Cli::parse();
    let bgg = Bgg::new();
    match &cli.command {
        Commands::Collection(args) => print_list(
            &bgg,
            bgg.collection(&args.user, true),
            &args.data,
            cli.verbose,
            &args.filter,
            &args.sort,
        ),
        Commands::Detail(args) => match bgg.detail(args.id) {
            Err(e) => print_err(e),
            Ok(game) => print_game(game, cli.verbose),
        },
        Commands::Search(args) => print_list(
            &bgg,
            bgg.search(&args.name),
            &Data::Games,
            cli.verbose,
            &None,
            &None,
        ),
    }
}
