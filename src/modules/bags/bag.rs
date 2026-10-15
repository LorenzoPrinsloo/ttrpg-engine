use std::collections::HashMap;
use std::fmt::Display;

use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::bags::math::greatest_common_denominator;
use crate::modules::config::BagFile;
use crate::modules::draw::Draw;

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    try_from = "BagFile",
    into = "BagFile",
    bound(
        deserialize = "Bag<T>: TryFrom<BagFile>, \
        <Bag<T> as TryFrom<BagFile>>::Error: Display",
        serialize = "Bag<T>:  Clone + Into<BagFile>"
    )
)]
pub struct Bag<T> {
    pub(crate) items: Vec<T>,
    pub(crate) distributions: HashMap<T, f64>,
    // are items in this bag unique (Have only 1 available and should remove the chance of finding one when one is discovered)
    unique: bool,
}

// Why a bag could not be built. Carries no entry position: the ratios come
// from a map, which has no order to point at. The loader adds the file name.
#[derive(Debug, PartialEq)]
pub enum BagError {
    // Ratios are quantized to hundredths, so the sum is carried as percent.
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
    // Fails rather than panics, because these ratios come from a file the
    // user edits by hand.
    pub(crate) fn try_new(
        distributions: HashMap<T, f64>,
        maybe_remaining_items: Option<HashMap<T, usize>>,
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

        let items: Vec<T> = match maybe_remaining_items {
            Some(remaining_items) => Self::load_items_from_remaining(&remaining_items),
            None => Self::starting_items(&distributions),
        };

        Ok(Self {
            items,
            distributions,
            unique,
        })
    }

    fn load_items_from_remaining(remaining: &HashMap<T, usize>) -> Vec<T> {
        remaining
            .iter()
            .flat_map(|(t, &v)| std::iter::repeat_n(t.clone(), v))
            .collect()
    }

    fn starting_items(distributions: &HashMap<T, f64>) -> Vec<T> {
        let parts: Vec<(&T, u32)> = distributions
            .iter()
            .map(|(v, ratio)| {
                assert!(
                    (0.0..=1.0).contains(ratio),
                    "all ratios must be between 0.0..=1.0"
                );
                (v, (ratio * 100.0).round() as u32)
            })
            .collect();

        // One divisor for the whole bag
        let divisor = parts
            .iter()
            .fold(0, |divisor, (_, part)| {
                greatest_common_denominator(divisor, *part)
            })
            .max(1);

        parts
            .iter()
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
    use std::collections::HashSet;

    use proptest::collection::vec;
    use proptest::strategy::Strategy;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    use super::*;

    fn copies<T: Eq>(bag: &Bag<T>, item: T) -> usize {
        bag.items.iter().filter(|held| **held == item).count()
    }

    #[test]
    fn ratios_that_do_not_add_up_are_reported() {
        let error = Bag::try_new(HashMap::from([("common", 0.5), ("rare", 0.3)]), None, false)
            .err()
            .unwrap();

        assert_eq!(error, BagError::Ratios(80));
        assert!(error.to_string().contains("0.8"), "{error}");
    }

    #[test]
    fn a_ratio_outside_zero_to_one_is_reported() {
        let error = Bag::try_new(HashMap::from([("impossible", 1.5)]), None, false)
            .err()
            .unwrap();

        assert_eq!(error, BagError::NotARatio(1.5));
    }

    #[test]
    fn copies_are_proportional_to_the_ratios() {
        let bag = Bag::try_new(
            HashMap::from([("common", 0.5), ("uncommon", 0.3), ("rare", 0.2)]),
            None,
            false,
        )
        .unwrap();

        // 50:30:20 reduces to 5:3:2.
        assert_eq!(copies(&bag, "common"), 5);
        assert_eq!(copies(&bag, "uncommon"), 3);
        assert_eq!(copies(&bag, "rare"), 2);
    }

    #[test]
    fn a_likelier_item_is_never_outnumbered_by_a_rarer_one() {
        let bag = Bag::try_new(
            HashMap::from([("common", 0.5), ("uncommon", 0.3), ("rare", 0.2)]),
            None,
            false,
        )
        .unwrap();

        assert!(copies(&bag, "common") > copies(&bag, "uncommon"));
        assert!(copies(&bag, "uncommon") > copies(&bag, "rare"));
    }

    // Splits 100 hundredths into between one and eight parts, so the ratios
    // always add up to 1.0, as a valid bag file's do.
    fn parts_of_a_hundred() -> impl Strategy<Value = Vec<u32>> {
        vec(1..=40_u32, 1..=8).prop_filter_map("needs room for every part", |weights| {
            let total: u32 = weights.iter().sum();
            let parts: Vec<u32> = weights.iter().map(|weight| weight * 100 / total).collect();
            let short = 100 - parts.iter().sum::<u32>();
            let parts: Vec<u32> = parts
                .iter()
                .enumerate()
                .map(|(i, part)| if i == 0 { part + short } else { *part })
                .collect();
            parts.iter().all(|part| *part > 0).then_some(parts)
        })
    }

    fn ratios(parts: &[u32]) -> HashMap<String, f64> {
        parts
            .iter()
            .enumerate()
            .map(|(i, part)| (format!("item {i}"), f64::from(*part) / 100.0))
            .collect()
    }

    fn sorted<T: Ord + Clone>(items: &[T]) -> Vec<T> {
        let mut items = items.to_vec();
        items.sort();
        items
    }

    proptest! {
        #[test]
        fn a_full_bag_holds_copies_in_the_ratio_proportions(parts in parts_of_a_hundred()) {
            let bag = Bag::try_new(ratios(&parts), None, false).unwrap();
            let total = bag.remaining();

            for (i, part) in parts.iter().enumerate() {
                let held = copies(&bag, format!("item {i}"));
                prop_assert_eq!(held * 100, *part as usize * total, "item {} of {:?}", i, parts);
            }
        }

        #[test]
        fn drawing_a_bag_empty_returns_exactly_its_marbles(parts in parts_of_a_hundred()) {
            let mut bag = Bag::try_new(ratios(&parts), None, false).unwrap();
            let marbles = sorted(&bag.items);

            let drawn: Vec<String> = std::iter::from_fn(|| bag.draw()).collect();

            prop_assert_eq!(sorted(&drawn), marbles);
            prop_assert_eq!(bag.remaining(), 0);
            prop_assert!(bag.draw().is_none());
        }

        #[test]
        fn each_draw_takes_exactly_one_marble_out(parts in parts_of_a_hundred()) {
            let mut bag = Bag::try_new(ratios(&parts), None, false).unwrap();
            let start = bag.remaining();

            for taken in 1..=start {
                bag.draw().unwrap();
                prop_assert_eq!(bag.remaining(), start - taken);
            }
        }

        #[test]
        fn a_unique_bag_never_repeats_an_item_before_a_refill(parts in parts_of_a_hundred()) {
            let mut bag = Bag::try_new(ratios(&parts), None, true).unwrap();

            let drawn: Vec<String> = std::iter::from_fn(|| bag.draw()).collect();
            let distinct: HashSet<&String> = drawn.iter().collect();

            prop_assert_eq!(distinct.len(), drawn.len(), "{:?}", drawn);
            prop_assert_eq!(drawn.len(), parts.len());
        }

        #[test]
        fn filling_a_bag_restores_its_starting_marbles(parts in parts_of_a_hundred(), draws in 0..50_usize) {
            let mut bag = Bag::try_new(ratios(&parts), None, true).unwrap();
            let marbles = sorted(&bag.items);

            for _ in 0..draws {
                bag.draw();
            }
            bag.fill();

            prop_assert_eq!(sorted(&bag.items), marbles);
        }

        #[test]
        fn ratios_that_do_not_add_up_to_one_are_refused(parts in vec(1..=60_u32, 1..=6)) {
            let total: u32 = parts.iter().sum();
            let result = Bag::try_new(ratios(&parts), None, false);

            if total == 100 {
                prop_assert!(result.is_ok());
            } else {
                prop_assert_eq!(result.err(), Some(BagError::Ratios(total)));
            }
        }
    }
}
