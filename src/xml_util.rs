use std::{fmt::Display, str::FromStr};

use roxmltree::Node;

use crate::{constants::*, error::Error};

pub fn attribute<T: Default + FromStr>(n: &Node, name: &str) -> Result<T, Error>
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

pub fn has_attribute(n: &Node, key: &str, value: &str) -> bool {
    match n.attribute(key) {
        None => false,
        Some(a) => a == value,
    }
}

pub fn node<'a>(n: &'a Node, name: &str) -> Result<Node<'a, 'a>, Error> {
    for child in n.children() {
        if child.is_element() && child.tag_name().name() == name {
            return Ok(child);
        }
    }
    Err(format!("node '{}' not found", name).into())
}

pub fn node_with_attr<'a>(
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

pub fn node_text<'a>(n: &'a Node, name: &str) -> Result<&'a str, Error> {
    node(n, name)?
        .text()
        .ok_or(format!("missing text for node '{}'", name).into())
}

pub fn get_id(n: &Node) -> Result<u32, Error> {
    match attribute(n, ATTR_OBJECT_ID) {
        Ok(id) => Ok(id),
        Err(_) => attribute(n, ATTR_ID),
    }
}

pub fn get_name(n: &Node) -> Result<String, Error> {
    match node_text(&n, TAG_NAME) {
        Ok(name) => Ok(name.to_string()),
        Err(_) => attribute(&node(&n, TAG_NAME)?, ATTR_VALUE),
    }
}

pub fn get_num_plays(n: &Node) -> Result<u32, Error> {
    match node_text(&n, TAG_NUM_PLAYS) {
        Ok(np) => Ok(np.parse::<u32>()?),
        Err(_) => Ok(0),
    }
}
