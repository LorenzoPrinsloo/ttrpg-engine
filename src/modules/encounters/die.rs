use serde::Deserialize;
use crate::encounters::Dice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Deserialize)]
pub struct Die {
    pub n: u32,
    pub sides: u32,
}

impl Die {
    pub fn roll(&self) -> u32 {
        self.n.d(&self.sides)
    }
}