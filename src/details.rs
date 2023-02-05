use std::fmt::{Display, Formatter};

use roxmltree::Node;

use crate::{constants::*, error::Error, xml_util::*};

#[derive(Debug)]
pub struct Details {
    pub id: u32,
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

impl Display for Details {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: rated {}", self.name, self.rating)?;
        if let Some(r) = self.rank {
            write!(f, " (ranked {})", r)?;
        }
        Ok(())
    }
}
