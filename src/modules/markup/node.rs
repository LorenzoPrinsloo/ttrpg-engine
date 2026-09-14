use crate::markup::DiceExpr;
use colored::Color;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;

/// A parsed markup tree. Styles are branches, text and dice are leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Text(String),
    Bold(Vec<Node>),
    Italic(Vec<Node>),
    Underline(Vec<Node>),
    Color { color: Color, children: Vec<Node> },
    Dice(DiceExpr),
}


/// Hand written because `colored::Color` is not `Hash`, and a tree stored in
/// an `Encounter` has to be hashable to key `Bag`'s distribution map.
impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
        match self {
            Self::Text(text) => text.hash(state),
            Self::Bold(children) | Self::Italic(children) | Self::Underline(children) => {
                children.hash(state)
            }
            Self::Dice(expr) => expr.hash(state),
            Self::Color { color, children } => {
                discriminant(color).hash(state);
                match color {
                    Color::AnsiColor(code) => code.hash(state),
                    Color::TrueColor { r, g, b } => (r, g, b).hash(state),
                    _ => {}
                }
                children.hash(state);
            }
        }
    }
}
