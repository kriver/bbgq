use std::{collections::HashMap, str::FromStr};

use reqwest::blocking::get;
use roxmltree::{Document, Node};
use url::Url;

const BGG: &str = "https://boardgamegeek.com/xmlapi2";

const PARAM_EXCL_SUBTYPE: &str = "excludesubtype";
const PARAM_ID: &str = "id";
const PARAM_STATS: &str = "stats";
const PARAM_USER_NAME: &str = "username";

const PATH_COLLECTION: &str = "collection";
const PATH_THING: &str = "thing";

const TAG_AVERAGE: &str = "average";
const TAG_LINK: &str = "link";
const TAG_NAME: &str = "name";
const TAG_NUM_PLAYS: &str = "numplays";
const TAG_RANK: &str = "rank";
const TAG_RANKS: &str = "ranks";
const TAG_RATINGS: &str = "ratings";
const TAG_STATISTICS: &str = "statistics";
const TAG_STATUS: &str = "status";

const ATTR_NAME: &str = "name";
const ATTR_OBJECT_ID: &str = "objectid";
const ATTR_OWN: &str = "own";
const ATTR_TYPE: &str = "type";
const ATTR_VALUE: &str = "value";

fn attribute<T: Default + FromStr>(n: &Node, name: &str) -> T {
    n.attribute(name)
        .map(|v| v.parse::<T>())
        .map(Result::ok)
        .flatten()
        .unwrap_or_default()
}

fn has_attribute(n: &Node, key: &str, value: &str) -> bool {
    match n.attribute(key) {
        None => false,
        Some(a) => a == value,
    }
}

fn node<'a>(n: &'a Node, name: &str) -> Option<Node<'a, 'a>> {
    for child in n.children() {
        if child.is_element() && child.tag_name().name() == name {
            return Some(child);
        }
    }
    None
}

fn node_with_attr<'a>(n: &'a Node, name: &str, key: &str, value: &str) -> Option<Node<'a, 'a>> {
    for child in n.children() {
        if child.is_element() && child.tag_name().name() == name {
            if has_attribute(&child, key, value) {
                return Some(child);
            }
        }
    }
    None
}

fn node_text<'a>(n: &'a Node, name: &str) -> Option<&'a str> {
    node(n, name).map(|n| n.text()).flatten()
}

fn item_status(n: &Node, attr: &str) -> bool {
    match node(n, TAG_STATUS) {
        Some(status) => status.attribute(attr).map(|b| b == "1").unwrap_or(false),
        None => false,
    }
}

#[derive(Debug)]
pub struct Game {
    id: u32,
    pub name: String,
    pub plays: u32,
    pub details: Option<Details>,
}

impl TryFrom<Node<'_, '_>> for Game {
    type Error = &'static str;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        let id = attribute(&n, ATTR_OBJECT_ID);
        match node_text(&n, TAG_NAME) {
            None => Err("name missing"),
            Some(name) => Ok(Game {
                id,
                name: name.to_string(),
                plays: node_text(&n, TAG_NUM_PLAYS)
                    .map(|v| v.parse::<u32>())
                    .map(Result::ok)
                    .flatten()
                    .unwrap_or(0),
                details: None,
            }),
        }
    }
}

#[derive(Debug)]
pub struct Details {
    pub rating: f32,
    pub rank: u32,
    pub categories: Vec<String>,
    pub mechanics: Vec<String>,
}

impl TryFrom<Node<'_, '_>> for Details {
    type Error = &'static str;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        fn ratings(n: &Node) -> (f32, u32) {
            let (mut rating, mut rank) = (0.0, 0);
            if let Some(a) = node(&n, TAG_AVERAGE) {
                rating = attribute(&a, ATTR_VALUE)
            }
            if let Some(r) = node(n, TAG_RANKS) {
                if let Some(r1) = node_with_attr(&r, TAG_RANK, ATTR_NAME, "boardgame") {
                    rank = attribute(&r1, ATTR_VALUE)
                }
            }
            (rating, rank)
        }

        let (mut rating, mut rank) = (0.0, 0);
        if let Some(s) = node(&n, TAG_STATISTICS) {
            if let Some(r) = node(&s, TAG_RATINGS) {
                (rating, rank) = ratings(&r);
            }
        };
        let categories = n
            .children()
            .filter(|c| c.tag_name().name() == TAG_LINK)
            .filter(|c| has_attribute(&c, ATTR_TYPE, "boardgamecategory"))
            .map(|c| attribute(&c, ATTR_VALUE))
            .collect();
        let mechanics = n
            .children()
            .filter(|c| c.tag_name().name() == TAG_LINK)
            .filter(|c| has_attribute(&c, ATTR_TYPE, "boardgamemechanic"))
            .map(|c| attribute(&c, ATTR_VALUE))
            .collect();
        Ok(Details {
            rating,
            rank,
            categories,
            mechanics,
        })
    }
}

pub struct Bgg {
    url: Url,
}

impl Bgg {
    pub fn new() -> Self {
        Bgg {
            url: Url::parse(BGG).unwrap(),
        }
    }

    fn request(&self, path: &str, params: HashMap<&str, &str>) -> Result<String, &'static str> {
        let mut url = self.url.clone();
        for (k, v) in params.into_iter() {
            url.query_pairs_mut().append_pair(k, v);
            url.path_segments_mut().unwrap().push(path);
        }
        match get(url.as_str()) {
            Err(_) => Err("failed to get data"),
            Ok(res) => match res.text() {
                Err(_) => Err("failed to read body"),
                Ok(body) => Ok(body),
            },
        }
    }

    pub fn collection(&self, user: &str, only_owned: bool) -> Result<Vec<Game>, &'static str> {
        let params = HashMap::from([
            (PARAM_USER_NAME, user),
            (PARAM_EXCL_SUBTYPE, "boardgameexpansion"),
        ]);
        match self.request(PATH_COLLECTION, params) {
            Err(e) => Err(e),
            Ok(body) => match Document::parse(&body) {
                Err(_) => Err("failed to parse XML"),
                Ok(xml) => Ok(xml
                    .root_element()
                    .children()
                    .into_iter()
                    .filter(Node::is_element)
                    .filter(|n| item_status(n, ATTR_OWN) == only_owned)
                    .map(TryFrom::try_from)
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
                    .collect()),
            },
        }
    }

    pub fn detail(&self, id: &u32) -> Result<Details, &'static str> {
        let id_str = id.to_string();
        let params = HashMap::from([(PARAM_ID, id_str.as_ref()), (PARAM_STATS, "1")]);
        match self.request(PATH_THING, params) {
            Err(e) => Err(e),
            Ok(body) => match Document::parse(&body) {
                Err(_) => Err("failed to parse XML"),
                Ok(xml) => xml
                    .root_element()
                    .children()
                    .into_iter()
                    .filter(Node::is_element)
                    .map(TryFrom::try_from)
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
                    .nth(0)
                    .ok_or("no details"),
            },
        }
    }

    pub fn fill_details(&self, game: &mut Game) -> Result<(), &'static str> {
        let id = game.id;
        match self.detail(&id) {
            Err(e) => Err(e),
            Ok(details) => {
                game.details = Some(details);
                Ok(())
            }
        }
    }
}
