use roxmltree::Node;

use crate::{constants::*, details::Details, error::Error, xml_util::*};

#[derive(Debug)]
pub struct Game {
    pub id: u32,
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
