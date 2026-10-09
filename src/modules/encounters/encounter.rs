use std::fmt::{Display, Formatter};

use crate::markup;
use crate::markup::{Node, parse};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Data {
    Markup(Vec<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Encounter {
    pub data: Data,
    pub(crate) raw_text: String,
}

impl Display for Encounter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.resolve().fmt(f)
    }
}

impl Encounter {
    pub fn markup(expr: &str) -> Self {
        Self {
            data: Data::Markup(parse(expr)),
            raw_text: expr.to_string(),
        }
    }

    pub fn resolve(&self) -> String {
        match self.data {
            Data::Markup(ref s) => markup::render_nodes(s),
        }
    }
}
