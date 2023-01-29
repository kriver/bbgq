use reqwest::blocking::get;
use roxmltree::{Document, Node};
use url::Url;

const BGG: &str = "https://boardgamegeek.com/xmlapi2";
const USER_NAME: &str = "username";
const EXCL_SUBTYPE: &str = "excludesubtype";

pub struct Bgg {
    url: Url,
}

pub struct Game {
    name: String,
    plays: u32,
}

// FIXME handle unwraps properly
// FIXME more constants for strings

impl Bgg {
    pub fn new() -> Self {
        Bgg {
            url: Url::parse(BGG).unwrap(),
        }
    }

    pub fn with_user(mut self, name: &str) -> Self {
        self.url.query_pairs_mut().append_pair(USER_NAME, name);
        self
    }

    fn named_node<'a>(node: &'a Node, name: &str) -> Option<Node<'a, 'a>> {
        for child in node.children() {
            if child.is_element() && child.tag_name().name() == name {
                return Some(child);
            }
        }
        None
    }

    pub fn collection(&self, only_owned: bool) -> Vec<Game> {
        let mut url = self.url.clone();
        url.query_pairs_mut()
            .append_pair(EXCL_SUBTYPE, "boardgameexpansion");
        url.path_segments_mut().unwrap().push("collection");
        let body = get(url.as_str()).unwrap().text().unwrap();
        let xml = Document::parse(&body).unwrap();
        for child in xml.root_element().children() {
            if !child.is_element() {
                continue;
            }
            if only_owned {
                let status = Bgg::named_node(&child, "status").unwrap();
                if !status.attribute("own").map(|v| v == "1").unwrap_or(false) {
                    continue;
                }
            }
            let name = Bgg::named_node(&child, "name").unwrap();
            println!("{}", name.text().unwrap());
        }
        vec![]
    }
}
