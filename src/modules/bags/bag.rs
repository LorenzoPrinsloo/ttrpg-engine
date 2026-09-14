use std::collections::HashMap;
use rand::RngExt;
use crate::modules::draw::Draw;
use crate::bags::math::greatest_common_denominator;

pub struct Bag<T> {
    items: Vec<T>,
    distributions: HashMap<T, f64>,
    // are items in this bag unique (Have only 1 available and should remove the chance of finding one when one is discovered)
    unique: bool
}

impl<T: Clone + Eq + std::hash::Hash> Bag<T> {
    pub(crate) fn new(distributions: HashMap<T, f64>, unique: bool) -> Self {
        let total_denominator: f64 = distributions.values()
            .into_iter()
            .map(|ratio| (ratio * 100.0).round())
            .sum();

        assert_eq!(total_denominator, 100.0, "the sum of all ratios provided must be exactly 1.0");

        let items: Vec<T> = Self::starting_items(&distributions);

        Self {
            items,
            distributions,
            unique
        }
    }

    fn starting_items(distributions: &HashMap<T, f64>) -> Vec<T> {
        let parts: Vec<(&T, u32)> = distributions.iter()
            .map(|(v, ratio)| {
                assert!((0.0..=1.0).contains(ratio), "all ratios must be between 0.0..=1.0");
                (v, (ratio * 100.0).round() as u32)
            })
            .collect();

        // One divisor for the whole bag
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
    fn copies_are_proportional_to_the_ratios() {
        let bag = Bag::new(
            HashMap::from([("common", 0.5), ("uncommon", 0.3), ("rare", 0.2)]),
            false,
        );

        // 50:30:20 reduces to 5:3:2.
        assert_eq!(copies(&bag, "common"), 5);
        assert_eq!(copies(&bag, "uncommon"), 3);
        assert_eq!(copies(&bag, "rare"), 2);
    }

    #[test]
    fn a_likelier_item_is_never_outnumbered_by_a_rarer_one() {
        let bag = Bag::new(
            HashMap::from([("common", 0.5), ("uncommon", 0.3), ("rare", 0.2)]),
            false,
        );

        assert!(copies(&bag, "common") > copies(&bag, "uncommon"));
        assert!(copies(&bag, "uncommon") > copies(&bag, "rare"));
    }
}