use reqwest::blocking::get;
use roxmltree::{Document, Node};
use url::Url;

const BGG: &str = "https://boardgamegeek.com/xmlapi2";

const PARAM_USER_NAME: &str = "username";
const PARAM_EXCL_SUBTYPE: &str = "excludesubtype";

const TAG_COLLECTION: &str = "collection";
const TAG_NAME: &str = "name";
const TAG_NUM_PLAYS: &str = "numplays";
const TAG_STATUS: &str = "status";

const ATTR_OWN: &str = "own";

fn node<'a>(n: &'a Node, name: &str) -> Option<Node<'a, 'a>> {
    for child in n.children() {
        if child.is_element() && child.tag_name().name() == name {
            return Some(child);
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
    pub name: String,
    pub plays: u32,
}

impl TryFrom<Node<'_, '_>> for Game {
    type Error = &'static str;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        match node_text(&n, TAG_NAME) {
            None => Err("name missing"),
            Some(name) => Ok(Game {
                name: name.to_string(),
                plays: node_text(&n, TAG_NUM_PLAYS)
                    .map(|v| v.parse::<u32>())
                    .map(Result::ok)
                    .flatten()
                    .unwrap_or(0),
            }),
        }
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

    pub fn with_user(mut self, name: &str) -> Self {
        self.url
            .query_pairs_mut()
            .append_pair(PARAM_USER_NAME, name);
        self
    }

    pub fn collection(&self, only_owned: bool) -> Result<Vec<Game>, &'static str> {
        let mut url = self.url.clone();
        url.query_pairs_mut()
            .append_pair(PARAM_EXCL_SUBTYPE, "boardgameexpansion");
        url.path_segments_mut().unwrap().push(TAG_COLLECTION);
        match get(url.as_str()) {
            Err(_) => Err("failed to get data"),
            Ok(res) => match res.text() {
                Err(_) => Err("failed to read body"),
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
            },
        }
    }
}
