use std::{collections::HashMap, fmt::Display, str::FromStr};

use reqwest::blocking::Client;
use roxmltree::{Document, Node};
use url::Url;

use crate::error::Error;

const CHUNK_SIZE: usize = 100;

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

const ATTR_ID: &str = "id";
const ATTR_NAME: &str = "name";
const ATTR_OBJECT_ID: &str = "objectid";
const ATTR_OWN: &str = "own";
const ATTR_TYPE: &str = "type";
const ATTR_VALUE: &str = "value";

const VALUE_NOT_RANKED: &str = "Not Ranked";

fn attribute<T: Default + FromStr>(n: &Node, name: &str) -> Result<T, Error>
where
    <T as FromStr>::Err: Display,
{
    match n.attribute(name) {
        None => Err(format!("attribute {} not found", name).into()),
        Some(v) => v
            .parse::<T>()
            .map_err(|e| Error::Message(format!("{} (value '{}')", e, v))),
    }
}

fn has_attribute(n: &Node, key: &str, value: &str) -> bool {
    match n.attribute(key) {
        None => false,
        Some(a) => a == value,
    }
}

fn node<'a>(n: &'a Node, name: &str) -> Result<Node<'a, 'a>, Error> {
    for child in n.children() {
        if child.is_element() && child.tag_name().name() == name {
            return Ok(child);
        }
    }
    Err(format!("node '{}' not found", name).into())
}

fn node_with_attr<'a>(
    n: &'a Node,
    name: &str,
    key: &str,
    value: &str,
) -> Result<Node<'a, 'a>, Error> {
    for child in n.children() {
        if child.is_element() && child.tag_name().name() == name {
            if has_attribute(&child, key, value) {
                return Ok(child);
            }
        }
    }
    Err(format!("node '{}' not found", name).into())
}

fn node_text<'a>(n: &'a Node, name: &str) -> Result<&'a str, Error> {
    node(n, name)?
        .text()
        .ok_or(format!("missing text for node '{}'", name).into())
}

#[derive(Debug)]
pub struct Game {
    id: u32,
    pub name: String,
    pub plays: u32,
    pub details: Option<Details>,
}

impl TryFrom<Node<'_, '_>> for Game {
    type Error = Error;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        let id = attribute(&n, ATTR_OBJECT_ID)?;
        let name = node_text(&n, TAG_NAME)?;
        let np = node_text(&n, TAG_NUM_PLAYS)?.parse::<u32>()?;
        Ok(Game {
            id,
            name: name.to_string(),
            plays: np,
            details: None,
        })
    }
}

#[derive(Debug)]
pub struct Details {
    id: u32,
    pub name: String,
    pub rating: f32,
    pub rank: Option<u32>,
    pub categories: Vec<String>,
    pub mechanics: Vec<String>,
}

impl TryFrom<Node<'_, '_>> for Details {
    type Error = Error;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        fn ratings(n: &Node) -> Result<(f32, Option<u32>), Error> {
            let rank = match attribute::<String>(
                &node_with_attr(&node(n, TAG_RANKS)?, TAG_RANK, ATTR_NAME, "boardgame")?,
                ATTR_VALUE,
            )? {
                r if r == VALUE_NOT_RANKED => None,
                r => Some(r.parse()?),
            };
            Ok((attribute(&node(&n, TAG_AVERAGE)?, ATTR_VALUE)?, rank))
        }

        let id = attribute(&n, ATTR_ID)?;
        let name = attribute(&node(&n, TAG_NAME)?, ATTR_VALUE)?;
        let (rating, rank) = ratings(&node(&node(&n, TAG_STATISTICS)?, TAG_RATINGS)?)?;
        let categories = n
            .children()
            .filter(|c| c.tag_name().name() == TAG_LINK)
            .filter(|c| has_attribute(&c, ATTR_TYPE, "boardgamecategory"))
            .map(|c| attribute(&c, ATTR_VALUE))
            .collect::<Result<Vec<String>, Error>>()?;
        let mechanics = n
            .children()
            .filter(|c| c.tag_name().name() == TAG_LINK)
            .filter(|c| has_attribute(&c, ATTR_TYPE, "boardgamemechanic"))
            .map(|c| attribute(&c, ATTR_VALUE))
            .collect::<Result<Vec<String>, Error>>()?;
        Ok(Details {
            id,
            name,
            rating,
            rank,
            categories,
            mechanics,
        })
    }
}

pub struct Bgg {
    client: Client,
    url: Url,
}

impl Bgg {
    pub fn new() -> Self {
        Bgg {
            client: Client::new(),
            url: Url::parse(BGG).unwrap(),
        }
    }

    fn request(&self, path: &str, params: HashMap<&str, &str>) -> Result<String, Error> {
        let mut url = self.url.clone();
        for (k, v) in params.into_iter() {
            url.query_pairs_mut().append_pair(k, v);
            url.path_segments_mut().unwrap().push(path);
        }
        let res = self.client.get(url.as_str()).send()?;
        match res.status().is_success() {
            true => Ok(res.text()?),
            false => Err(format!("HTTP {}", res.status()).into()),
        }
    }

    pub fn collection(&self, user: &str, only_owned: bool) -> Result<Vec<Game>, Error> {
        fn item_status(n: &Node, attr: &str) -> bool {
            match node(n, TAG_STATUS) {
                Ok(status) => status.attribute(attr).map(|b| b == "1").unwrap_or(false),
                Err(_) => false,
            }
        }

        let params = HashMap::from([
            (PARAM_USER_NAME, user),
            (PARAM_EXCL_SUBTYPE, "boardgameexpansion"),
        ]);
        let body = self.request(PATH_COLLECTION, params)?;
        let xml = Document::parse(&body)?;
        xml.root_element()
            .children()
            .into_iter()
            .filter(Node::is_element)
            .filter(|n| item_status(n, ATTR_OWN) == only_owned)
            .map(TryFrom::try_from)
            .collect()
    }

    fn details(&self, ids: &[u32]) -> Result<Vec<Details>, Error> {
        let id_val = ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let params = HashMap::from([(PARAM_ID, id_val.as_ref()), (PARAM_STATS, "1")]);
        let body = self.request(PATH_THING, params)?;
        let xml = Document::parse(&body)?;
        xml.root_element()
            .children()
            .into_iter()
            .filter(Node::is_element)
            .map(TryFrom::try_from)
            .collect()
    }

    pub fn detail(&self, id: u32) -> Result<Details, Error> {
        self.details(&[id]).map(|mut details| details.remove(0))
    }

    pub fn fill_details(&self, games: &mut [Game]) -> Result<(), Error> {
        let ids: Vec<u32> = games.iter().map(|g| g.id).collect();
        let mut details = HashMap::new();
        for chunk in ids.chunks(CHUNK_SIZE) {
            for d in self.details(chunk)?.into_iter() {
                details.insert(d.id, d);
            }
        }
        for g in games.iter_mut() {
            g.details = details.remove(&g.id);
        }
        Ok(())
    }
}
