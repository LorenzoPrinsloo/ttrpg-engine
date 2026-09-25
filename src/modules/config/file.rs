use crate::modules::config::range::RowRange;
use serde::Deserialize;
use crate::bags::{Bag, BagError};
use crate::encounters::Encounter;
use crate::modules::tables::{Table, TableError};
/* A bag, from `tables/bags/<name>.toml`. Ratios are fractions of the whole
   bag and have to add up to 1.0. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BagFile {
    #[serde(default)]
    pub unique: bool,
    pub items: Vec<ItemEntry>,
}

impl TryFrom<BagFile> for Bag<Encounter> {
    type Error = BagError;

    fn try_from(file: BagFile) -> Result<Self, Self::Error> {
        let items = file
            .items
            .iter()
            .map(|item|
                (
                    Encounter::markup(&item.text.strip_suffix('\n').unwrap_or(&item.text)),
                    item.ratio
                )
            )
            .collect();

        Bag::try_new(items, file.unique)
    }
}

/* A table, from `tables/tables/<name>.toml`. Ranges are inclusive and must
   not overlap. */
#[derive(Debug, Deserialize)]
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
            .map(|row| (row.range.span(), Encounter::markup(&row.text.strip_suffix('\n').unwrap_or(&row.text))))
            .collect();

        Table::try_new(rows, file.unique)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemEntry {
    pub ratio: f64,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowEntry {
    pub range: RowRange,
    pub text: String,
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
