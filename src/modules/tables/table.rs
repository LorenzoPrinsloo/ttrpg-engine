use std::collections::HashMap;
use std::ops::RangeInclusive;
use rand::RngExt;
use crate::modules::draw::Draw;

pub struct Table<T> {
    items: HashMap<RangeInclusive<u32>, Item<T>>,
    max: u32,
    min: u32,
    // are items in this bag unique (Have only 1 available and should remove the chance of finding one when one is discovered)
    unique: bool
}

pub struct Item<T> {
    value: T,
    available: bool
}

impl<T> Item<T> {
    fn new(value: T, available: bool) -> Self {
        Item { value, available }
    }
}

/// Why a table could not be built. The loader adds the file name.
#[derive(Debug, PartialEq)]
pub enum TableError {
    Overlap(RangeInclusive<u32>, RangeInclusive<u32>),
    Empty,
}

impl std::fmt::Display for TableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overlap(earlier, later) => {
                write!(f, "rows {earlier:?} and {later:?} overlap")
            }
            Self::Empty => write!(f, "a table needs at least one row"),
        }
    }
}

impl std::error::Error for TableError {}

impl<T: Clone + Eq + std::hash::Hash> Table<T> {
    pub(crate) fn new(distributions: HashMap<RangeInclusive<u32>, T>, unique: bool) -> Self {
        Self::try_new(distributions, unique).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Fails rather than panics, because these rows come from a file the user
    /// edits by hand.
    pub(crate) fn try_new(
        distributions: HashMap<RangeInclusive<u32>, T>,
        unique: bool,
    ) -> Result<Self, TableError> {
        if let Some((earlier, later)) = Self::find_overlap(distributions.keys().collect()) {
            return Err(TableError::Overlap(earlier.clone(), later.clone()));
        }

        let min: u32 = *distributions.keys().map(|r| r.start()).min().ok_or(TableError::Empty)?;
        let max: u32 = *distributions.keys().map(|r| r.end()).max().ok_or(TableError::Empty)?;

        Ok(Table {
            items: distributions.into_iter()
                .map(|(k, v)| (k, Item::new(v, true)))
                .collect(),
            max,
            min,
            unique
        })
    }

    fn find_overlap(ranges: Vec<&RangeInclusive<u32>>) -> Option<(&RangeInclusive<u32>, &RangeInclusive<u32>)> {
        let mut sorted: Vec<&RangeInclusive<u32>> = ranges.iter().copied().collect();
        sorted.sort_by_key(|r| r.start());

        sorted.windows(2).find_map(|w| {
            let (a, b) = (w[0], w[1]);
            (b.start() <= a.end()).then_some((a, b))
        })
    }

}

impl<T: Clone + Eq + std::hash::Hash> Draw for Table<T> {
    type Item = T;

    fn draw(&mut self) -> Option<T> {
        if self.items.values().all(|v| !v.available) {
            return None;
        }
        loop {
            let i = rand::rng().random_range(self.min..=self.max);
            if let Some((_, item)) = self.items.iter_mut().find(|(r, item)| r.contains(&i) && item.available) {
                let t: Option<T> = if self.unique {
                    item.available = false;
                    Some(item.value.clone())
                } else { Some(item.value.clone()) };
                if t.is_some() {
                    return t;
                }
            }
        }
    }

    fn fill(&mut self) {
        self.items.iter_mut().for_each(|(_, v)| v.available = true);
    }

    fn remaining(&self) -> usize {
        self.items.iter().filter(|(_, v)| v.available).count()
    }

    fn unique(&self) -> bool {
        self.unique
    }

    /// Rows are only taken out of play when the table is unique; otherwise
    /// every draw is a fresh roll over all of them.
    fn exhausts(&self) -> bool {
        self.unique
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // -- helpers ------------------------------------------------------------

    fn table(entries: Vec<(RangeInclusive<u32>, &str)>, unique: bool) -> Table<String> {
        Table::new(
            entries.into_iter().map(|(r, s)| (r, s.to_string())).collect(),
            unique,
        )
    }

    // -- construction / validation ------------------------------------------

    #[test]
    #[should_panic(expected = "overlap")]
    fn overlapping_ranges_panic() {
        table(vec![(1..=10, "a"), (5..=15, "b")], false);
    }

    #[test]
    #[should_panic(expected = "overlap")]
    fn adjacent_inclusive_ranges_overlap() {
        // 5 is in BOTH 1..=5 and 5..=10 — must be rejected.
        // Fails with `<` in find_overlap; requires `<=` for RangeInclusive.
        table(vec![(1..=5, "a"), (5..=10, "b")], false);
    }

    #[test]
    fn overlapping_rows_name_both_ranges() {
        let error = Table::try_new(HashMap::from([(1..=10, "a"), (5..=15, "b")]), false)
            .err().unwrap();
        let message = error.to_string();

        assert!(message.contains("overlap"), "{message}");
        assert!(message.contains("1..=10") && message.contains("5..=15"), "{message}");
    }

    #[test]
    fn an_empty_table_is_rejected() {
        let error = Table::<&str>::try_new(HashMap::new(), false).err().unwrap();

        assert_eq!(error, TableError::Empty);
    }

    #[test]
    fn touching_but_disjoint_ranges_are_fine() {
        // 1..=5 and 6..=10 share no number — must be accepted.
        table(vec![(1..=5, "a"), (6..=10, "b")], false);
    }

    #[test]
    fn single_row_table() {
        let mut t = table(vec![(1..=100, "only")], false);
        assert_eq!(t.draw(), Some("only".to_string()));
    }

    // -- draw: non-unique ----------------------------------------------------

    #[test]
    fn non_unique_draw_never_exhausts() {
        let mut t = table(vec![(1..=2, "a"), (3..=4, "b")], false);
        for _ in 0..1000 {
            assert!(t.draw().is_some(), "non-unique table must always yield");
        }
    }

    #[test]
    fn draw_yields_only_table_contents() {
        let mut t = table(vec![(1..=50, "a"), (51..=100, "b")], false);
        for _ in 0..500 {
            let item = t.draw().unwrap();
            assert!(item == "a" || item == "b", "drew a value not in the table: {item}");
        }
    }

    // -- draw: unique ---------------------------------------------------------

    #[test]
    fn unique_table_yields_each_item_exactly_once_then_none() {
        let mut t = table(
            vec![(1..=25, "a"), (26..=50, "b"), (51..=75, "c"), (76..=100, "d")],
            true,
        );
        let mut seen = HashSet::new();
        while let Some(item) = t.draw() {
            assert!(seen.insert(item.clone()), "duplicate from unique table: {item}");
        }
        assert_eq!(seen.len(), 4, "unique table must yield every item before exhausting");
        assert_eq!(t.draw(), None, "exhausted table must keep returning None");
        assert_eq!(t.draw(), None);
    }

    // -- boundaries -----------------------------------------------------------

    #[test]
    fn zero_based_and_offset_tables_drain_fully() {
        // min/max derived from data: both conventions must work unchanged
        for entries in [
            vec![(0..=4, "a"), (5..=9, "b")],     // 0-based
            vec![(78..=79, "a"), (80..=81, "b")], // offset, d100-style rows
        ] {
            let mut t = table(entries, true);
            let mut n = 0;
            while t.draw().is_some() {
                n += 1;
                assert!(n <= 2, "drew more items than the table holds");
            }
            assert_eq!(n, 2);
        }
    }

    #[test]
    fn gapped_table_still_terminates() {
        // rolls in 11..=89 hit no range; reroll loop must absorb them
        let mut t = table(vec![(1..=10, "a"), (90..=100, "b")], true);
        assert!(t.draw().is_some());
        assert!(t.draw().is_some());
        assert_eq!(t.draw(), None);
    }

    // -- distribution (statistical) --------------------------------------------

    #[test]
    fn draw_respects_weights_roughly() {
        // 80/20 split over many draws from a fresh non-unique table
        let mut t = table(vec![(1..=80, "common"), (81..=100, "rare")], false);
        let draws = 10_000;
        let commons = (0..draws)
            .filter(|_| t.draw().as_deref() == Some("common"))
            .count();
        let ratio = commons as f64 / draws as f64;
        assert!(
            (0.76..=0.84).contains(&ratio),
            "expected ~0.80 common, got {ratio}"
        );
    }
}