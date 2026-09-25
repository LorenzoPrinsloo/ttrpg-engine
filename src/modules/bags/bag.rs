use std::collections::HashMap;
use rand::RngExt;
use crate::modules::draw::Draw;
use crate::bags::math::greatest_common_denominator;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(
    try_from = "crate::modules::config::BagFile",
    bound(deserialize = "Bag<T>: TryFrom<crate::modules::config::BagFile>, \
        <Bag<T> as TryFrom<crate::modules::config::BagFile>>::Error: std::fmt::Display")
)]
pub struct Bag<T> {
    items: Vec<T>,
    distributions: HashMap<T, f64>,
    /* are items in this bag unique (Have only 1 available and should remove the chance of finding one when one is discovered) */
    unique: bool
}

/* Why a bag could not be built. Carries no entry position: the ratios come
   from a map, which has no order to point at. The loader adds the file name. */
#[derive(Debug, PartialEq)]
pub enum BagError {
    /* Ratios are quantized to hundredths, so the sum is carried as percent. */
    Ratios(u32),
    NotARatio(f64),
}

impl std::fmt::Display for BagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ratios(percent) => write!(
                f,
                "the ratios add up to {}, they have to add up to 1.0",
                f64::from(*percent) / 100.0
            ),
            Self::NotARatio(ratio) => {
                write!(f, "a ratio of {ratio} is not between 0.0 and 1.0")
            }
        }
    }
}

impl std::error::Error for BagError {}

impl<T: Clone + Eq + std::hash::Hash> Bag<T> {
    /* Fails rather than panics, because these ratios come from a file the
       user edits by hand. */
    pub(crate) fn try_new(
        distributions: HashMap<T, f64>,
        unique: bool,
    ) -> Result<Self, BagError> {
        let mut percent = 0;
        for ratio in distributions.values() {
            if !(0.0..=1.0).contains(ratio) {
                return Err(BagError::NotARatio(*ratio));
            }
            percent += (ratio * 100.0).round() as u32;
        }

        if percent != 100 {
            return Err(BagError::Ratios(percent));
        }

        Ok(Self {
            items: Self::starting_items(&distributions),
            distributions,
            unique,
        })
    }

    fn starting_items(distributions: &HashMap<T, f64>) -> Vec<T> {
        let parts: Vec<(&T, u32)> = distributions.iter()
            .map(|(v, ratio)| {
                assert!((0.0..=1.0).contains(ratio), "all ratios must be between 0.0..=1.0");
                (v, (ratio * 100.0).round() as u32)
            })
            .collect();

        /* One divisor for the whole bag */
        let divisor = parts.iter()
            .fold(0, |divisor, (_, part)| greatest_common_denominator(divisor, *part))
            .max(1);

        parts.iter()
            .flat_map(|(v, part)| std::iter::repeat_n((*v).clone(), (part / divisor) as usize))
            .collect()
    }

}

impl<T: Clone + Eq + std::hash::Hash> Draw for Bag<T> {
    type Item = T;

    fn draw(&mut self) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }
        let i = rand::rng().random_range(0..self.items.len());
        let drawn = self.items.swap_remove(i);

        if self.unique {
            self.items.retain(|item| item != &drawn);
        }

        Some(drawn)
    }

    fn fill(&mut self) {
        self.items.clear();
        self.items.extend(Self::starting_items(&self.distributions));
    }

    fn remaining(&self) -> usize {
        self.items.len()
    }

    fn unique(&self) -> bool {
        self.unique
    }

    fn exhausts(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copies<T: Eq>(bag: &Bag<T>, item: T) -> usize {
        bag.items.iter().filter(|held| **held == item).count()
    }

    #[test]
    fn ratios_that_do_not_add_up_are_reported() {
        let error = Bag::try_new(HashMap::from([("common", 0.5), ("rare", 0.3)]), false)
            .err().unwrap();

        assert_eq!(error, BagError::Ratios(80));
        assert!(error.to_string().contains("0.8"), "{error}");
    }

    #[test]
    fn a_ratio_outside_zero_to_one_is_reported() {
        let error = Bag::try_new(HashMap::from([("impossible", 1.5)]), false).err().unwrap();

        assert_eq!(error, BagError::NotARatio(1.5));
    }

    #[test]
    fn copies_are_proportional_to_the_ratios() {
        let bag = Bag::try_new(
            HashMap::from([("common", 0.5), ("uncommon", 0.3), ("rare", 0.2)]),
            false,
        )
        .unwrap();

        /* 50:30:20 reduces to 5:3:2. */
        assert_eq!(copies(&bag, "common"), 5);
        assert_eq!(copies(&bag, "uncommon"), 3);
        assert_eq!(copies(&bag, "rare"), 2);
    }

    #[test]
    fn a_likelier_item_is_never_outnumbered_by_a_rarer_one() {
        let bag = Bag::try_new(
            HashMap::from([("common", 0.5), ("uncommon", 0.3), ("rare", 0.2)]),
            false,
        )
        .unwrap();

        assert!(copies(&bag, "common") > copies(&bag, "uncommon"));
        assert!(copies(&bag, "uncommon") > copies(&bag, "rare"));
    }
}