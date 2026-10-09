use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::bags::{Bag, BagError};
use crate::encounters::Encounter;
use crate::modules::config::range::RowRange;
use crate::modules::draw::Draw;
use crate::modules::tables::{Table, TableError};
// A bag, from `tables/bags/<name>.toml`. Ratios are fractions of the whole
// bag and have to add up to 1.0.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BagFile {
    #[serde(default)]
    pub unique: bool,
    pub items: Vec<ItemEntry>,
}

impl TryFrom<BagFile> for Bag<Encounter> {
    type Error = BagError;

    fn try_from(file: BagFile) -> Result<Self, Self::Error> {
        let items = file.items.iter().map(|item| {
            (
                Encounter::markup(&item.text.strip_suffix('\n').unwrap_or(&item.text)),
                (item.ratio, item.remaining),
            )
        });

        let item_ratios: HashMap<Encounter, f64> = items.clone().map(|(v, r)| (v, r.0)).collect();

        // The full bag is what a config file gets, and it supplies the count
        // for any entry whose `remaining` is left out.
        let full = Bag::try_new(item_ratios.clone(), None, file.unique)?;
        if file.items.iter().all(|item| item.remaining.is_none()) {
            return Ok(full);
        }

        let items_remaining: HashMap<Encounter, usize> = items
            .map(|(v, r)| {
                let left =
                    r.1.unwrap_or_else(|| full.items.iter().filter(|e| **e == v).count());
                (v, left)
            })
            .collect();

        Bag::try_new(item_ratios, Some(items_remaining), file.unique)
    }
}

impl From<Bag<Encounter>> for BagFile {
    fn from(bag: Bag<Encounter>) -> Self {
        let unique = bag.unique();
        let mut items: Vec<ItemEntry> = bag
            .distributions
            .into_iter()
            .map(|(item, rat)| ItemEntry {
                ratio: rat,
                text: item.raw_text.clone(),
                remaining: Some(bag.items.iter().filter(|e| **e == item).count()),
            })
            .collect();

        // HashMap order changes from run to run; sorted, an unchanged bag
        // saves the same file every time.
        items.sort_by(|a, b| a.text.cmp(&b.text));

        Self { unique, items }
    }
}

// A table, from `tables/tables/<name>.toml`. Ranges are inclusive and must
// not overlap.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowsFile {
    #[serde(default)]
    pub unique: bool,
    pub rows: Vec<RowEntry>,
}

impl TryFrom<RowsFile> for Table<Encounter> {
    type Error = TableError;

    fn try_from(file: RowsFile) -> Result<Self, Self::Error> {
        let rows = file
            .rows
            .iter()
            .map(|row| {
                (
                    row.range.span(),
                    (
                        Encounter::markup(&row.text.strip_suffix('\n').unwrap_or(&row.text)),
                        row.used.unwrap_or(false),
                    ),
                )
            })
            .collect();

        Table::try_new(rows, file.unique)
    }
}

impl From<Table<Encounter>> for RowsFile {
    fn from(value: Table<Encounter>) -> Self {
        let unique = value.unique();
        let mut rows: Vec<RowEntry> = value
            .items
            .into_iter()
            .map(|(range, item)| RowEntry {
                range: range.into(),
                text: item.value.raw_text,
                used: Some(!item.available),
            })
            .collect();

        // In range order, as the table reads in a book.
        rows.sort_by_key(|row| *row.range.span().start());

        Self { unique, rows }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemEntry {
    pub ratio: f64,
    pub text: String,
    // Copies left, counted after the ratios are reduced by their gcd: 0.3 in
    // a 0.1-step bag is at most 3. Save files write it; left out, the entry
    // starts full.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowEntry {
    pub range: RowRange,
    pub text: String,
    // Save files write it; left out, the row is still in play.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used: Option<bool>,
}

#[cfg(test)]
mod tests {
    use figment::Figment;
    use figment::providers::{Format, Toml};
    use serde::de::DeserializeOwned;

    use super::*;

    fn parse<T: DeserializeOwned>(source: &str) -> Result<T, figment::Error> {
        Figment::from(Toml::string(source)).extract()
    }

    #[test]
    fn unique_defaults_to_false() {
        let file: BagFile = parse("[[items]]\nratio = 1.0\ntext = 'x'").unwrap();

        assert!(!file.unique);
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn a_bag_file_needs_items() {
        let error = parse::<BagFile>("unique = true").err().unwrap().to_string();

        assert!(error.contains("items"), "{error}");
    }

    #[test]
    fn a_bag_cannot_hold_rows() {
        let error = parse::<BagFile>("[[rows]]\nrange = '1-2'\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("rows"), "{error}");
    }

    #[test]
    fn a_table_cannot_hold_items() {
        let error = parse::<RowsFile>("[[items]]\nratio = 1.0\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("items"), "{error}");
    }

    #[test]
    fn a_misspelled_key_is_rejected_by_name() {
        let error = parse::<BagFile>("[[items]]\nratoi = 1.0\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        // figment points at the entry by key path rather than by line.
        assert!(error.contains("items.0.ratoi"), "{error}");
    }

    #[test]
    fn a_top_level_key_below_the_first_entry_is_rejected() {
        // The trap worth a test: after [[items]], `unique` would belong to
        // that entry, which `deny_unknown_fields` catches.
        let error = parse::<BagFile>("[[items]]\nratio = 1.0\ntext = 'x'\nunique = true")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("unique"), "{error}");
    }

    #[test]
    fn rows_carry_a_range() {
        let file: RowsFile = parse("[[rows]]\nrange = '01-19'\ntext = 'x'").unwrap();

        assert_eq!(file.rows[0].range.span(), 1..=19);
    }

    #[test]
    fn a_bad_range_reports_its_entry() {
        let error = parse::<RowsFile>("[[rows]]\nrange = '19-1'\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("backwards"), "{error}");
        assert!(error.contains("rows.0.range"), "{error}");
    }

    #[test]
    fn an_entry_without_a_count_starts_full() {
        // 0.75:0.25 is 3:1 copies. `common` is drawn down to 1, `rare` states
        // nothing and keeps its 1.
        let file: BagFile = parse(
            "[[items]]\nratio = 0.75\ntext = 'common'\nremaining = 1\n\
             [[items]]\nratio = 0.25\ntext = 'rare'",
        )
        .unwrap();

        let bag = Bag::try_from(file).unwrap();

        assert_eq!(bag.remaining(), 2);
    }

    #[test]
    fn saved_entries_come_out_in_a_fixed_order() {
        let bag = Bag::try_new(
            HashMap::from([
                (Encounter::markup("c"), 0.5),
                (Encounter::markup("a"), 0.25),
                (Encounter::markup("b"), 0.25),
            ]),
            None,
            false,
        )
        .unwrap();
        let table = Table::try_new(
            HashMap::from([
                (20..=20, (Encounter::markup("z"), false)),
                (1..=9, (Encounter::markup("y"), false)),
                (10..=19, (Encounter::markup("x"), false)),
            ]),
            false,
        )
        .unwrap();

        let texts: Vec<String> = BagFile::from(bag)
            .items
            .into_iter()
            .map(|item| item.text)
            .collect();
        let ranges: Vec<String> = RowsFile::from(table)
            .rows
            .iter()
            .map(|row| row.range.to_string())
            .collect();

        assert_eq!(texts, ["a", "b", "c"]);
        assert_eq!(ranges, ["1-9", "10-19", "20"]);
    }
}
