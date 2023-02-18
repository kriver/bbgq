use std::fmt::{Display, Formatter};

use roxmltree::Node;

use crate::{details::Details, error::Error, xml_util::*};

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
        write!(f, "{}: id={}, plays={}", self.name, self.id, self.plays)?;
        if let Some(d) = self.details.as_ref() {
            write!(f, ", {}", d)?;
        }
        Ok(())
    }
}
