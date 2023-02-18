use std::fmt::{Display, Formatter};

use roxmltree::Node;

use crate::{constants::*, details::Details, error::Error, xml_util::*};

#[derive(Debug)]
pub struct Game {
    pub id: u32,
    pub name: String,
    pub plays: u32,
    pub details: Option<Details>,
}

fn get_id(n: &Node) -> Result<u32, Error> {
    match attribute(n, ATTR_OBJECT_ID) {
        Ok(id) => Ok(id),
        Err(_) => attribute(n, ATTR_ID),
    }
}

fn get_name(n: &Node) -> Result<String, Error> {
    match node_text(&n, TAG_NAME) {
        Ok(name) => Ok(name.to_string()),
        Err(_) => attribute(&node(&n, TAG_NAME)?, ATTR_VALUE),
    }
}

fn get_num_plays(n: &Node) -> Result<u32, Error> {
    match node_text(&n, TAG_NUM_PLAYS) {
        Ok(np) => Ok(np.parse::<u32>()?),
        Err(_) => Ok(0),
    }
}

impl TryFrom<Node<'_, '_>> for Game {
    type Error = Error;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        let id = get_id(&n)?;
        let name = get_name(&n)?;
        let np = get_num_plays(&n)?;
        Ok(Game {
            id,
            name,
            plays: np,
            details: None,
        })
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.details.as_ref() {
            None => write!(f, "{}", self.name),
            Some(d) => write!(f, "{}", d),
        }
    }
}
