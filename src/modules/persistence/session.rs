use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::bags::Bag;
use crate::encounters::Encounter;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::draw::Draw;
    use std::collections::{BTreeSet, HashMap};

    fn session() -> SessionFile {
        let bag = Bag::try_new(
            HashMap::from([
                (Encounter::markup("**common**"), 0.75),
                (Encounter::markup("rare"), 0.25),
            ]),
            None,
            true,
        )
        .unwrap();

        let table = Table::try_new(
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
}