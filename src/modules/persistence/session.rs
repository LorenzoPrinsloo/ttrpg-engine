use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::bags::Bag;
use crate::encounters::Encounter;
use crate::modules::config::Tables;
use crate::modules::config::hash_key;
use crate::modules::tables::Table;

#[derive(Serialize, Deserialize)]
pub(crate) struct SessionFile {
    pub(crate) version: u32,
    pub(crate) config_hashes: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) bags: BTreeMap<String, Bag<Encounter>>,
    #[serde(default)]
    pub(crate) tables: BTreeMap<String, Table<Encounter>>,
}

impl SessionFile {
    /* The newest format this build reads. A file saying more came from a
       newer build, and may hold fields this one would drop on the next save. */
    pub(crate) const VERSION: u32 = 1;

    /* What gets written to disk: every bag and table as it stands, with the
       fingerprints of the files they were loaded from. */
    pub(crate) fn capture(
        bags: &[(String, Bag<Encounter>)],
        tables: &[(String, Table<Encounter>)],
        hashes: &BTreeMap<String, String>,
    ) -> Self {
        SessionFile {
            version: Self::VERSION,
            config_hashes: hashes.clone(),
            bags: bags.iter().cloned().collect(),
            tables: tables.iter().cloned().collect(),
        }
    }

    /* Lays the saved state over a freshly loaded config. A bag or table only
       comes back if its file is byte for byte what it was at the save: an
       edited file may have new entries or ratios the saved one knows nothing
       of, so it starts full instead. Returns a note for each one that did not
       come back, for the user to read. */
    pub(crate) fn restore(self, fresh: &mut Tables) -> Vec<String> {
        let mut notes = Vec::new();

        restore_section("bags", self.bags, &mut fresh.bags, &self.config_hashes, &fresh.hashes, &mut notes);
        restore_section("tables", self.tables, &mut fresh.tables, &self.config_hashes, &fresh.hashes, &mut notes);

        notes
    }
}

fn restore_section<S>(
    section: &str,
    saved: BTreeMap<String, S>,
    fresh: &mut BTreeMap<String, S>,
    saved_hashes: &BTreeMap<String, String>,
    fresh_hashes: &BTreeMap<String, String>,
    notes: &mut Vec<String>,
) {
    for (name, state) in saved {
        let key = hash_key(section, &name);

        let Some(slot) = fresh.get_mut(&name) else {
            notes.push(format!("{key} is gone, the saved {name} is dropped"));
            continue;
        };

        match (saved_hashes.get(&key), fresh_hashes.get(&key)) {
            (Some(saved), Some(fresh)) if saved == fresh => *slot = state,
            _ => notes.push(format!("{key} changed since the last save, {name} starts full")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::draw::Draw;
    use std::collections::{BTreeSet, HashMap};

    fn session() -> SessionFile {
        let bag: Bag<Encounter> = Bag::try_new(
            HashMap::from([
                (Encounter::markup("**common**"), 0.75),
                (Encounter::markup("rare"), 0.25),
            ]),
            None,
            true,
        )
        .unwrap();

        let table: Table<Encounter> = Table::try_new(
            HashMap::from([
                (1..=19, (Encounter::markup("static"), false)),
                (20..=20, (Encounter::markup("*patrol*"), false)),
            ]),
            false,
        )
        .unwrap();

        SessionFile {
            version: 1,
            config_hashes: BTreeMap::from([
                ("bags/artifacts.toml".to_string(), "abc123".to_string()),
                ("tables/rumours.toml".to_string(), "def456".to_string()),
            ]),
            bags: BTreeMap::from([("artifacts".to_string(), bag)]),
            tables: BTreeMap::from([("rumours".to_string(), table)]),
        }
    }

    #[test]
    fn test_parse_session() {
        let json = serde_json::to_string(&session()).unwrap();
        let back: SessionFile = serde_json::from_str(&json).unwrap();

        assert_eq!(back.version, 1);
        assert_eq!(back.config_hashes, session().config_hashes);

        /* The bag comes back through try_new: same entries by their source
           text, the same 3:1 copies, and `unique` kept. */
        let bag = &back.bags["artifacts"];
        let texts: BTreeSet<&str> = bag.distributions.keys().map(|e| e.raw_text.as_str()).collect();
        assert_eq!(texts, BTreeSet::from(["**common**", "rare"]));
        assert_eq!(bag.remaining(), 4);
        assert!(bag.unique());

        /* Ranges survive as their "1-19" / "20" spelling and back. */
        let table = &back.tables["rumours"];
        let rows: BTreeSet<(u32, u32, &str)> = table
            .items
            .iter()
            .map(|(range, item)| (*range.start(), *range.end(), item.value.raw_text.as_str()))
            .collect();
        assert_eq!(rows, BTreeSet::from([(1, 19, "static"), (20, 20, "*patrol*")]));
        assert!(!table.unique());
    }

    fn round_trip(session: &SessionFile) -> SessionFile {
        serde_json::from_str(&serde_json::to_string(session).unwrap()).unwrap()
    }

    #[test]
    fn a_depleted_bag_stays_depleted() {
        let mut session = session();
        let bag = session.bags.get_mut("artifacts").unwrap();

        /* Unique, so one draw takes every copy of whichever entry came up:
           4 copies become 1 (common drawn) or 3 (rare drawn). */
        let drawn = bag.draw().unwrap();
        let left = bag.remaining();
        assert!(left < 4);

        let back = round_trip(&session);
        let bag = &back.bags["artifacts"];

        assert_eq!(bag.remaining(), left);
        assert!(!bag.items.contains(&drawn), "{} came back", drawn.raw_text);
    }

    #[test]
    fn a_used_row_stays_used() {
        let table = Table::try_new(
            HashMap::from([
                (1..=10, (Encounter::markup("first"), false)),
                (11..=20, (Encounter::markup("second"), false)),
            ]),
            true,
        )
        .unwrap();
        let mut session = SessionFile {
            tables: BTreeMap::from([("rumours".to_string(), table)]),
            ..session()
        };

        let drawn = session.tables.get_mut("rumours").unwrap().draw().unwrap();

        let mut back = round_trip(&session);
        let table = back.tables.get_mut("rumours").unwrap();

        /* The row that came up is still used: the only draw left is the other
           one, and then the table is empty. */
        assert_eq!(table.remaining(), 1);
        assert_ne!(table.draw().unwrap(), drawn, "{} came back available", drawn.raw_text);
        assert_eq!(table.draw(), None);
    }

    /* -- Restoring over a fresh config ---------------------------------------- */

    /* What a start loads: the session's bag and table as the files say, full,
       with `hash` as the fingerprint of both files. */
    fn fresh(hash: &str) -> Tables {
        let saved = session();
        Tables {
            bags: saved.bags.clone(),
            tables: saved.tables.clone(),
            hashes: BTreeMap::from([
                ("bags/artifacts.toml".to_string(), hash.to_string()),
                ("tables/rumours.toml".to_string(), hash.to_string()),
            ]),
        }
    }

    /* The session after a draw from the bag, saved against `hash`. */
    fn played(hash: &str) -> SessionFile {
        let mut tables = fresh(hash);
        tables.bags.get_mut("artifacts").unwrap().draw().unwrap();

        let bags: Vec<_> = tables.bags.into_iter().collect();
        let rows: Vec<_> = tables.tables.into_iter().collect();
        round_trip(&SessionFile::capture(&bags, &rows, &tables.hashes))
    }

    #[test]
    fn an_unchanged_file_gets_its_saved_state_back() {
        let mut tables = fresh("same");

        let notes = played("same").restore(&mut tables);

        assert!(notes.is_empty(), "{notes:?}");
        assert!(tables.bags["artifacts"].remaining() < 4);
    }

    #[test]
    fn an_edited_file_starts_full_and_says_so() {
        let mut tables = fresh("edited");

        let notes = played("before").restore(&mut tables);

        assert_eq!(tables.bags["artifacts"].remaining(), 4);
        assert!(notes.iter().any(|note| note.contains("bags/artifacts.toml changed")), "{notes:?}");
    }

    #[test]
    fn a_file_that_is_gone_drops_its_saved_state() {
        let mut tables = fresh("same");
        tables.bags.clear();

        let notes = played("same").restore(&mut tables);

        assert!(tables.bags.is_empty());
        assert!(notes.iter().any(|note| note.contains("bags/artifacts.toml is gone")), "{notes:?}");
    }

    #[test]
    fn a_new_file_starts_full_without_a_note() {
        let mut tables = fresh("same");
        let mut saved = played("same");
        saved.bags.clear();

        let notes = saved.restore(&mut tables);

        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(tables.bags["artifacts"].remaining(), 4);
    }
}
