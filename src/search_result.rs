use std::fmt::{Display, Formatter};

use roxmltree::Node;

use crate::{constants::*, error::Error, xml_util::*};

#[derive(Debug)]
pub struct SearchResult {
    pub id: u32,
    pub name: String,
}

impl TryFrom<Node<'_, '_>> for SearchResult {
    type Error = Error;

    fn try_from(n: Node<'_, '_>) -> Result<Self, Self::Error> {
        let id = attribute(&n, ATTR_ID)?;
        let name = attribute(&node(&n, TAG_NAME)?, ATTR_VALUE)?;
        Ok(SearchResult { id, name })
    }
}

impl Display for SearchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
