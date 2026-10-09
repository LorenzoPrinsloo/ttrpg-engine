use std::collections::BTreeMap;
use crate::modules::bags::Bag;
use crate::modules::config::error::LoadError;
use crate::modules::config::paths;
use crate::modules::encounters::Encounter;
use crate::modules::tables::Table;
use std::path::{Path, PathBuf};
use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;
use crate::config::provider::Nested;
use crate::modules::config::hash;

#[derive(Deserialize)]
pub struct Tables {
    #[serde(default)]
    pub bags: BTreeMap<String, Bag<Encounter>>,
    #[serde(default)]
    pub tables: BTreeMap<String, Table<Encounter>>,
    /* Each file's fingerprint, keyed by its path under the root, as
       `bags/artifacts.toml`. Not part of the document: filled in beside it. */
    #[serde(skip)]
    pub hashes: BTreeMap<String, String>,
}

/* The key a file's fingerprint is kept under, both here and in a saved
   session: `bags/artifacts.toml` for the bag called "artifacts". */
pub(crate) fn hash_key(section: &str, name: &str) -> String {
    format!("{section}/{name}.toml")
}

pub fn load() -> Result<(PathBuf, Tables), Vec<LoadError>> {
    let root: PathBuf = paths::resolve().map_err(|error| vec![error])?;
    let tables = load_from(&root)?;

    Ok((root, tables))
}

/* Everything under `root`, which is taken as found: no searching. */
pub(crate) fn load_from(root: &Path) -> Result<Tables, Vec<LoadError>> {
    let mut figment: Figment = Figment::new();
    let mut errors: Vec<LoadError> = Vec::new();
    let mut hashes: BTreeMap<String, String> = BTreeMap::new();

    for section in ["bags", "tables"] {
        for path in toml_files(&root.join(section)) {
            let name = match name_of(&path) {
                Ok(name) => name,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };

            /* Read apart from figment, which opens the file itself. An edit
               landing between the two reads would go unnoticed until the next
               start, which is fine for a file edited by hand. */
            match std::fs::read(&path) {
                Ok(bytes) => {
                    hashes.insert(hash_key(section, &name), hash::fingerprint(&bytes));
                }
                Err(error) => {
                    errors.push(LoadError::Io { path, error });
                    continue;
                }
            }

            figment = figment.merge(Nested::new(section, name, Toml::file_exact(path)));
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut tables: Tables = figment
        .extract()
        .map_err(|error| error.into_iter().map(LoadError::Config).collect::<Vec<_>>())?;

    if tables.bags.is_empty() && tables.tables.is_empty() {
        return Err(vec![LoadError::NoTables { root: root.to_path_buf() }]);
    }

    tables.hashes = hashes;

    Ok(tables)
}

fn toml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .collect()
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

    #[test]
    fn a_syntax_error_reports_the_line() {
        let error = tables("[[bags.artifacts.items]\nratio = 1.0").err().unwrap().to_string();

        assert!(error.contains("line"), "{error}");
    }

    #[test]
    fn a_broken_file_is_named_in_the_error() {
        let dir = std::env::temp_dir().join(format!("untitled-loader-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("artifacts.toml");
        std::fs::write(&path, "[[items]]\nratio = 0.97\ntext = 'x'").unwrap();

        let error = Figment::from(Nested::new("bags", "artifacts", Toml::file_exact(&path)))
            .extract::<Tables>()
            .err()
            .unwrap()
            .to_string();
        std::fs::remove_dir_all(&dir).ok();

        assert!(error.contains("artifacts.toml"), "{error}");
        assert!(error.contains("0.97"), "{error}");
    }

    #[test]
    fn an_embedded_newline_survives_but_a_trailing_one_does_not() {
        let mut loaded =
            tables("[[bags.notes.items]]\nratio = 1.0\ntext = '''\nfirst\nsecond\n'''").unwrap();
        let drawn = loaded.bags.get_mut("notes").unwrap().draw().unwrap();

        assert_eq!(drawn.resolve(), "first\nsecond");
    }

    #[test]
    fn a_name_with_a_space_is_rejected() {
        let error = name_of(Path::new("config/bags/loot table.toml")).err().unwrap();

        assert!(error.to_string().contains("cannot be empty or contain spaces"));
    }

    const ENCOUNTERS_TOML: &str = include_str!("../../../config/bags/encounters.toml");
    const ARTIFACTS_TOML: &str = include_str!("../../../config/bags/artifacts.toml");

    #[test]
    fn the_shipped_files_have_no_backslashes() {
        /* Markup owns `\`, so any that appeared would mean TOML escaping had
           crept into text that is meant to be literal. */
        assert!(!ENCOUNTERS_TOML.contains('\\'));
        assert!(!ARTIFACTS_TOML.contains('\\'));
    }

    #[test]
    fn the_shipped_bags_keep_their_shape() {
        let loaded: Tables = Figment::new()
            .merge(Nested::new("bags", "encounters", Toml::string(ENCOUNTERS_TOML)))
            .merge(Nested::new("bags", "artifacts", Toml::string(ARTIFACTS_TOML)))
            .extract()
            .unwrap();

        /* 1:3:3:2:1 after the gcd, and a true d100 table. */
        assert_eq!(loaded.bags["encounters"].remaining(), 10);
        assert_eq!(loaded.bags["artifacts"].remaining(), 100);
        assert!(loaded.bags["artifacts"].unique());
    }

    #[test]
    fn every_file_is_fingerprinted_under_its_path() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("bags")).unwrap();
        std::fs::create_dir_all(root.path().join("tables")).unwrap();
        std::fs::write(root.path().join("bags/artifacts.toml"), ARTIFACTS_TOML).unwrap();
        std::fs::write(root.path().join("tables/rumours.toml"), "[[rows]]\nrange = '1'\ntext = 'x'").unwrap();

        let loaded = load_from(root.path()).unwrap();

        assert_eq!(
            loaded.hashes,
            BTreeMap::from([
                ("bags/artifacts.toml".to_string(), hash::fingerprint(ARTIFACTS_TOML.as_bytes())),
                ("tables/rumours.toml".to_string(), hash::fingerprint(b"[[rows]]\nrange = '1'\ntext = 'x'")),
            ])
        );
    }

    #[test]
    fn a_missing_directory_holds_no_files() {
        assert!(toml_files(Path::new("/nonexistent/bags")).is_empty());
    }

    /* -- Tables through figment ---------------------------------------------- */

    /* The nested shape the per-file providers merge into: every file's contents
       under `bags.<name>` or `tables.<name>`. */
    fn tables(source: &str) -> Result<Tables, figment::Error> {
        Figment::from(Toml::string(source)).extract()
    }

    #[test]
    fn named_bags_and_tables_land_under_their_names() {
        let loaded = tables(
            "[bags.artifacts]\nunique = true\n\
             [[bags.artifacts.items]]\nratio = 1.0\ntext = 'relic'\n\
             [bags.encounters]\n\
             [[bags.encounters.items]]\nratio = 0.75\ntext = 'common'\n\
             [[bags.encounters.items]]\nratio = 0.25\ntext = 'rare'\n\
             [tables.rumours]\n\
             [[tables.rumours.rows]]\nrange = '01-19'\ntext = 'static'\n\
             [[tables.rumours.rows]]\nrange = '20'\ntext = 'patrol'",
        )
        .unwrap();

        assert_eq!(loaded.bags.keys().collect::<Vec<_>>(), ["artifacts", "encounters"]);
        assert_eq!(loaded.tables.keys().collect::<Vec<_>>(), ["rumours"]);

        /* Reaching try_new shows up as the ratios reduced and `unique` kept. */
        assert!(loaded.bags["artifacts"].unique());
        assert_eq!(loaded.bags["encounters"].remaining(), 4);
        assert_eq!(loaded.tables["rumours"].remaining(), 2);
    }

    #[test]
    fn a_missing_section_is_empty() {
        let loaded = tables("[[bags.artifacts.items]]\nratio = 1.0\ntext = 'relic'").unwrap();

        assert_eq!(loaded.bags.len(), 1);
        assert!(loaded.tables.is_empty());
    }

    #[test]
    fn nothing_at_all_is_two_empty_sections() {
        let loaded = tables("").unwrap();

        assert!(loaded.bags.is_empty());
        assert!(loaded.tables.is_empty());
    }

    #[test]
    fn bag_validation_runs_during_extraction() {
        let error = tables("[[bags.artifacts.items]]\nratio = 0.97\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("0.97"), "{error}");
        assert!(error.contains("bags.artifacts"), "{error}");
    }

    #[test]
    fn table_validation_runs_during_extraction() {
        let error = tables(
            "[[tables.rumours.rows]]\nrange = '1-10'\ntext = 'a'\n\
             [[tables.rumours.rows]]\nrange = '5-15'\ntext = 'b'",
        )
        .err()
        .unwrap()
        .to_string();

        assert!(error.contains("overlap"), "{error}");
        assert!(error.contains("tables.rumours"), "{error}");
    }

    #[test]
    fn a_bad_range_is_reported_under_its_table() {
        let error = tables("[[tables.rumours.rows]]\nrange = '19-1'\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("backwards"), "{error}");
        assert!(error.contains("tables.rumours"), "{error}");
    }

    #[test]
    fn a_misspelled_key_in_an_entry_is_rejected() {
        let error = tables("[[bags.artifacts.items]]\nratoi = 1.0\ntext = 'x'")
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("ratoi"), "{error}");
    }

    #[test]
    fn entry_text_is_parsed_as_markup() {
        let mut loaded = tables(
            "[[bags.artifacts.items]]\nratio = 1.0\ntext = '''\n**[3d1] Black Marble(s).**\n'''",
        )
        .unwrap();

        /* Compared as parsed markup rather than rendered text: rendering depends
           on `colored`'s global switch, which the render tests turn on while
           this one may be running. Equal to the untrimmed text's parse means
           the trailing newline is gone too. */
        let drawn = loaded.bags.get_mut("artifacts").unwrap().draw().unwrap();
        assert_eq!(drawn, Encounter::markup("**[3d1] Black Marble(s).**"));
    }
}
