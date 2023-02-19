use std::collections::HashMap;

use reqwest::blocking::Client;
use roxmltree::{Document, Node};
use url::Url;

use crate::{constants::*, error::Error, game::Game, xml_util::node};

const CHUNK_SIZE: usize = 100;

const BGG: &str = "https://boardgamegeek.com/xmlapi2";

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

    fn details(&self, ids: &[u32]) -> Result<Vec<Game>, Error> {
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
            .map(|n| {
                let mut game: Game = n.try_into()?;
                game.details = Some(n.try_into()?);
                Ok(game)
            })
            .collect()
    }

    pub fn detail(&self, id: u32) -> Result<Game, Error> {
        self.details(&[id]).map(|mut details| details.remove(0))
    }

    pub fn fill_details(&self, mut games: Vec<Game>) -> Result<Vec<Game>, Error> {
        let ids: Vec<u32> = games.iter().map(|g| g.id).collect();
        let mut details = HashMap::new();
        for chunk in ids.chunks(CHUNK_SIZE) {
            for g in self.details(chunk)?.into_iter() {
                details.insert(g.id, g.details.unwrap());
            }
        }
        for g in games.iter_mut() {
            g.details = details.remove(&g.id);
        }
        Ok(games)
    }

    pub fn search(&self, name: &str) -> Result<Vec<Game>, Error> {
        let params = HashMap::from([(PARAM_QUERY, name), (PARAM_TYPE, "boardgame")]);
        let body = self.request(PATH_SEARCH, params)?;
        let xml = Document::parse(&body)?;
        xml.root_element()
            .children()
            .into_iter()
            .filter(Node::is_element)
            .map(TryFrom::try_from)
            .collect()
    }
}
