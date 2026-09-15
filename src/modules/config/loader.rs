use crate::modules::bags::Bag;
use crate::modules::config::error::LoadError;
use crate::modules::config::file::{BagFile, RowsFile};
use crate::modules::config::paths;
use crate::modules::encounters::Encounter;
use crate::modules::tables::Table;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

pub struct Tables {
    pub bags: Vec<(String, Bag<Encounter>)>,
    pub tables: Vec<(String, Table<Encounter>)>,
}

pub fn load() -> Result<(PathBuf, Tables), Vec<LoadError>> {
    let root = paths::resolve().map_err(|error| vec![error])?;

    let (bags, mut errors) = load_dir(&root.join("bags"), bag_from_str);
    let (tables, table_errors) = load_dir(&root.join("tables"), table_from_str);
    errors.extend(table_errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    if bags.is_empty() && tables.is_empty() {
        return Err(vec![LoadError::NoTables { root }]);
    }

    Ok((root, Tables { bags, tables }))
}

pub(crate) fn bag_from_str(path: &Path, source: &str) -> Result<Bag<Encounter>, LoadError> {
    let file: BagFile = parse(path, source)?;
    let items = file
        .items
        .iter()
        .map(|item| (Encounter::markup(entry_text(&item.text)), item.ratio))
        .collect();

    Bag::try_new(items, file.unique).map_err(|error| LoadError::Bag { path: path.into(), error })
}

pub(crate) fn table_from_str(path: &Path, source: &str) -> Result<Table<Encounter>, LoadError> {
    let file: RowsFile = parse(path, source)?;
    let rows = file
        .rows
        .iter()
        .map(|row| (row.range.span(), Encounter::markup(entry_text(&row.text))))
        .collect();

    Table::try_new(rows, file.unique).map_err(|error| LoadError::Table { path: path.into(), error })
}

fn parse<T: DeserializeOwned>(path: &Path, source: &str) -> Result<T, LoadError> {
    toml::from_str(source).map_err(|error| LoadError::Syntax { path: path.into(), error })
}

fn entry_text(raw: &str) -> &str {
    raw.strip_suffix('\n').unwrap_or(raw)
}

fn load_dir<T>(
    dir: &Path,
    parse: fn(&Path, &str) -> Result<T, LoadError>,
) -> (Vec<(String, T)>, Vec<LoadError>) {
    let mut loaded = Vec::new();
    let mut errors = Vec::new();

    let Ok(entries) = std::fs::read_dir(dir) else {
        return (loaded, errors);
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .collect();

    paths.sort();

    for path in paths {
        let loading = name_of(&path).and_then(|name| {
            let source = std::fs::read_to_string(&path)
                .map_err(|error| LoadError::Unreadable { path: path.clone(), error })?;

            parse(&path, &source).map(|value| (name, value))
        });

        match loading {
            Ok(named) => loaded.push(named),
            Err(error) => errors.push(error),
        }
    }

    (loaded, errors)
}

fn name_of(path: &Path) -> Result<String, LoadError> {
    let name = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default();

    if name.is_empty() || name.contains(char::is_whitespace) {
        return Err(LoadError::Name { path: path.into() });
    }

    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::draw::Draw;

    fn bag(source: &str) -> Result<Bag<Encounter>, LoadError> {
        bag_from_str(Path::new("tables/bags/artifacts.toml"), source)
    }

    fn table(source: &str) -> Result<Table<Encounter>, LoadError> {
        table_from_str(Path::new("tables/tables/rumours.toml"), source)
    }

    #[test]
    fn a_bag_becomes_items_in_proportion() {
        let loaded = bag(
            "[[items]]\nratio = 0.75\ntext = 'common'\n\n[[items]]\nratio = 0.25\ntext = 'rare'",
        )
        .unwrap();

        /* 75:25 reduces to 3:1. */
        assert_eq!(loaded.remaining(), 4);
    }

    #[test]
    fn a_table_keeps_one_row_per_entry() {
        let loaded = table(
            "[[rows]]\nrange = '01-19'\ntext = 'static'\n\n[[rows]]\nrange = '20'\ntext = 'patrol'",
        )
        .unwrap();

        assert_eq!(loaded.remaining(), 2);
    }

    #[test]
    fn unique_carries_through() {
        let loaded = bag("unique = true\n\n[[items]]\nratio = 1.0\ntext = 'relic'").unwrap();

        assert!(loaded.unique());
    }

    #[test]
    fn ratios_that_do_not_add_up_name_the_file_and_the_sum() {
        let error = bag("[[items]]\nratio = 0.97\ntext = 'x'").err().unwrap().to_string();

        assert!(error.contains("artifacts.toml"), "{error}");
        assert!(error.contains("0.97"), "{error}");
    }

    #[test]
    fn overlapping_rows_name_the_file() {
        let source = "[[rows]]\nrange = '1-10'\ntext = 'a'\n\n[[rows]]\nrange = '5-15'\ntext = 'b'";
        let error = table(source).err().unwrap().to_string();

        assert!(error.contains("rumours.toml"), "{error}");
        assert!(error.contains("overlap"), "{error}");
    }

    #[test]
    fn a_syntax_error_names_the_file_and_the_line() {
        let error = bag("[[items]\nratio = 1.0").err().unwrap().to_string();

        assert!(error.contains("artifacts.toml"), "{error}");
        assert!(error.contains("line"), "{error}");
    }

    #[test]
    fn markup_in_an_entry_is_parsed_not_stored_raw() {
        let loaded = bag("[[items]]\nratio = 1.0\ntext = '**[3d1] Black Marble(s).**'").unwrap();
        let mut loaded = loaded;

        /* Dice roll at render time; d1 keeps it deterministic. */
        assert_eq!(loaded.draw().unwrap().resolve(), "3 Black Marble(s).");
    }

    #[test]
    fn an_embedded_newline_survives_but_a_trailing_one_does_not() {
        let source = "[[items]]\nratio = 1.0\ntext = '''\nfirst\nsecond\n'''";
        let mut loaded = bag(source).unwrap();

        assert_eq!(loaded.draw().unwrap().resolve(), "first\nsecond");
    }

    #[test]
    fn a_name_with_a_space_is_rejected() {
        let error = name_of(Path::new("tables/bags/loot table.toml")).err().unwrap();

        assert!(error.to_string().contains("cannot be empty or contain spaces"));
    }

    const ENCOUNTERS_TOML: &str = include_str!("../../../tables/bags/encounters.toml");
    const ARTIFACTS_TOML: &str = include_str!("../../../tables/bags/artifacts.toml");

    #[test]
    fn the_shipped_files_have_no_backslashes() {
        /* Markup owns `\`, so any that appeared would mean TOML escaping had
           crept into text that is meant to be literal. */
        assert!(!ENCOUNTERS_TOML.contains('\\'));
        assert!(!ARTIFACTS_TOML.contains('\\'));
    }

    #[test]
    fn the_shipped_bags_keep_their_shape() {
        let encounters = bag_from_str(Path::new("encounters.toml"), ENCOUNTERS_TOML).unwrap();
        let artifacts = bag_from_str(Path::new("artifacts.toml"), ARTIFACTS_TOML).unwrap();

        /* 1:3:3:2:1 after the gcd, and a true d100 table. */
        assert_eq!(encounters.remaining(), 10);
        assert_eq!(artifacts.remaining(), 100);
        assert!(artifacts.unique());
    }

    #[test]
    fn a_missing_directory_is_not_an_error_on_its_own() {
        let (loaded, errors) = load_dir(Path::new("/nonexistent/bags"), bag_from_str);

        assert!(loaded.is_empty());
        assert!(errors.is_empty());
    }
}
