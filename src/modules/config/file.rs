use crate::modules::config::range::RowRange;
use serde::Deserialize;

/* A bag, from `tables/bags/<name>.toml`. Ratios are fractions of the whole
   bag and have to add up to 1.0. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BagFile {
    #[serde(default)]
    pub unique: bool,
    pub items: Vec<ItemEntry>,
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

    #[test]
    fn unique_defaults_to_false() {
        let file: BagFile = toml::from_str("[[items]]\nratio = 1.0\ntext = 'x'").unwrap();

        assert!(!file.unique);
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn a_bag_file_needs_items() {
        let error = toml::from_str::<BagFile>("unique = true").err().unwrap().to_string();

        assert!(error.contains("items"), "{error}");
    }

    #[test]
    fn a_bag_cannot_hold_rows() {
        let error = toml::from_str::<BagFile>("[[rows]]\nrange = '1-2'\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("rows"), "{error}");
    }

    #[test]
    fn a_table_cannot_hold_items() {
        let error = toml::from_str::<RowsFile>("[[items]]\nratio = 1.0\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("items"), "{error}");
    }

    #[test]
    fn a_misspelled_key_is_rejected_by_name() {
        let error = toml::from_str::<BagFile>("[[items]]\nratoi = 1.0\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("ratoi"), "{error}");
        assert!(error.contains("line"), "{error}");
    }

    #[test]
    fn a_top_level_key_below_the_first_entry_is_rejected() {
        /* The trap worth a test: after [[items]], `unique` would belong to
           that entry, which `deny_unknown_fields` catches. */
        let error = toml::from_str::<BagFile>("[[items]]\nratio = 1.0\ntext = 'x'\nunique = true")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("unique"), "{error}");
    }

    #[test]
    fn rows_carry_a_range() {
        let file: RowsFile = toml::from_str("[[rows]]\nrange = '01-19'\ntext = 'x'").unwrap();

        assert_eq!(file.rows[0].range.span(), 1..=19);
    }

    #[test]
    fn a_bad_range_reports_the_line() {
        let error = toml::from_str::<RowsFile>("[[rows]]\nrange = '19-1'\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("backwards"), "{error}");
        assert!(error.contains("line"), "{error}");
    }
}
