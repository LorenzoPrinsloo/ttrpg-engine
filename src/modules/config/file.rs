use std::collections::HashMap;
use crate::modules::config::range::RowRange;
use serde::{Deserialize, Serialize};
use crate::bags::{Bag, BagError};
use crate::encounters::Encounter;
use crate::modules::draw::Draw;
use crate::modules::tables::{Table, TableError};
/* A bag, from `tables/bags/<name>.toml`. Ratios are fractions of the whole
   bag and have to add up to 1.0. */
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
        let items = file.items
            .iter()
            .map(|item|
                (
                    Encounter::markup(&item.text.strip_suffix('\n').unwrap_or(&item.text)),
                    (item.ratio, item.remaining)
                )
            );

        let item_ratios: HashMap<Encounter, f64> = items.clone().map(|( v, r)| (v,r.0)).collect();
        /* Some only when every entry states its count (a save file); config
           files state none, so they get None and a full bag. */
        let items_remaining: Option<HashMap<Encounter, usize>> = items.map(|( v, r)| r.1.map(|n| (v, n))).collect();

        Bag::try_new(item_ratios,  items_remaining, file.unique)
    }
}

impl From<Bag<Encounter>> for BagFile {
    fn from(bag: Bag<Encounter>) -> Self {
        Self {
            unique: bag.unique(),
            items: bag.distributions.into_iter().map(|(item, rat)|
                ItemEntry {
                    ratio: rat,
                    text: item.raw_text.clone(),
                    remaining: Some(bag.items.iter().filter(|e| **e == item).count())
                }
            ).collect(),
        }
    }
}

/* A table, from `tables/tables/<name>.toml`. Ranges are inclusive and must
   not overlap. */
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
            .map(|row| 
                (
                    row.range.span(),
                    (Encounter::markup(&row.text.strip_suffix('\n').unwrap_or(&row.text)), row.used.unwrap_or(false))
                )
            )
            .collect();

        Table::try_new(rows, file.unique)
    }
}

impl From<Table<Encounter>> for RowsFile {
    fn from(value: Table<Encounter>) -> Self {
        Self {
            unique: value.unique(),
            rows: value.items.into_iter()
                .map(|(range, item)|
                    RowEntry {
                        range: range.into(),
                        text: item.value.raw_text,
                        used: Some(!item.available)
                    }
                ).collect()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemEntry {
    pub ratio: f64,
    pub text: String,
    /* Copies left, counted after the ratios are reduced by their gcd: 0.3 in
       a 0.1-step bag is at most 3. Save files write it; left out, the entry
       starts full. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowEntry {
    pub range: RowRange,
    pub text: String,
    /* Save files write it; left out, the row is still in play. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::Figment;
    use figment::providers::{Format, Toml};
    use serde::de::DeserializeOwned;

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

        /* figment points at the entry by key path rather than by line. */
        assert!(error.contains("items.0.ratoi"), "{error}");
    }

    #[test]
    fn a_top_level_key_below_the_first_entry_is_rejected() {
        /* The trap worth a test: after [[items]], `unique` would belong to
           that entry, which `deny_unknown_fields` catches. */
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
}
