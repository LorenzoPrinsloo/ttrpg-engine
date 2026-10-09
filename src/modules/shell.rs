use std::io::{self, BufRead, Write};

use colored::Colorize;

use crate::modules::bags::Bag;
use crate::modules::config::Tables;
use crate::modules::draw::Draw;
use crate::modules::encounters::Encounter;
use crate::modules::markup::{self, DiceExpr};
use crate::modules::tables::Table;

type NamedBag = (String, Bag<Encounter>);
type NamedTable = (String, Table<Encounter>);

const HELP: &str = "\
**tables**            the tables loaded and how much is left in each
**draw** *table* [n]  draw n items, default 1; table names can be abbreviated
**refill** [*table*]  put everything back, every table if none is named
**reset** [*table*]   back to how its file has it, every table if none is named
**roll** *2d6+3*      roll a dice expression
**render** *markup*   preview a markup string
**quit**              leave, as does ctrl-D";

// Runs a session over `bags` and `tables`. With `args` it runs them as a
// single command and exits; otherwise it prompts until `quit` or end of input.
// `original` is the config as its files have it, before any saved session
// was laid over it, for `reset` to go back to. `save` is handed the bags and
// tables after every command that changed them, so quitting any way at all
// loses nothing.
pub fn run(
    bags: Vec<NamedBag>,
    tables: Vec<NamedTable>,
    original: Tables,
    args: &[String],
    mut save: impl FnMut(&[NamedBag], &[NamedTable]),
) {
    let mut session = Session::new(bags, tables, original);

    if !args.is_empty() {
        session.command(&args.join(" "));
        session.save_if_changed(&mut save);
        return;
    }

    println!(
        "{}",
        markup::render("**Mothership tables.** *help* lists the commands.")
    );
    session.list();

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("> ");
        io::stdout().flush().ok();

        match lines.next() {
            Some(Ok(line)) => {
                let flow = session.command(&line);
                session.save_if_changed(&mut save);

                if flow == Flow::Quit {
                    break;
                }
            }
            Some(Err(error)) => {
                eprintln!("{error}");
                break;
            }
            // ctrl-D: the newline leaves the terminal prompt on its own line.
            None => {
                println!();
                break;
            }
        }
    }
}

#[derive(PartialEq)]
enum Flow {
    Continue,
    Quit,
}

struct Session {
    bags: Vec<NamedBag>,
    tables: Vec<NamedTable>,
    original: Tables,
    // Whether a draw, refill or reset reached anything since the last save.
    changed: bool,
}

// The one bag or table a name resolved to. Borrowing it here rather than
// handing back an index keeps the two lists from being mixed up.
enum Found<'a> {
    Bag(&'a str, &'a mut Bag<Encounter>),
    Table(&'a str, &'a mut Table<Encounter>),
}

impl Session {
    // Names address everything in the session, so a repeated one would leave
    // an entry unreachable.
    fn new(bags: Vec<NamedBag>, tables: Vec<NamedTable>, original: Tables) -> Self {
        let mut names: Vec<&str> = bags.iter().map(|(name, _)| name.as_str()).collect();
        names.extend(tables.iter().map(|(name, _)| name.as_str()));
        names.sort_unstable();

        if let Some(pair) = names.windows(2).find(|pair| pair[0] == pair[1]) {
            panic!(
                "two tables are called {:?}, names have to be unique",
                pair[0]
            );
        }

        Session {
            bags,
            tables,
            original,
            changed: false,
        }
    }

    fn save_if_changed(&mut self, save: &mut impl FnMut(&[NamedBag], &[NamedTable])) {
        if std::mem::take(&mut self.changed) {
            save(&self.bags, &self.tables);
        }
    }

    fn command(&mut self, line: &str) -> Flow {
        let line = line.trim();
        let (head, rest) = line.split_once(' ').unwrap_or((line, ""));
        let rest = rest.trim();

        match head {
            "" => {}
            "help" | "?" => println!("{}", markup::render(HELP)),
            "tables" | "ls" => self.list(),
            "draw" | "d" => self.draw(rest),
            "refill" => self.refill(rest),
            "reset" => self.reset(rest),
            "roll" => roll(rest),
            "render" => println!("{}", markup::render(rest)),
            "quit" | "exit" | "q" => return Flow::Quit,
            _ => println!("unknown command {head:?}, try `help`"),
        }

        Flow::Continue
    }

    fn list(&self) {
        println!("Bags");
        for (name, bag) in &self.bags {
            println!("{}", row(name, bag));
        }

        println!("Tables");
        for (name, table) in &self.tables {
            println!("{}", row(name, table));
        }
    }

    // `draw <name> [count]`
    fn draw(&mut self, rest: &str) {
        let (name, count) = split_count(rest);

        match self.resolve(name) {
            Some(Found::Bag(name, bag)) => draw_from(name, bag, count),
            Some(Found::Table(name, table)) => draw_from(name, table, count),
            None => return,
        }
        self.changed = true;
    }

    // `refill [name]`, or everything when no name is given.
    fn refill(&mut self, name: &str) {
        if name.is_empty() {
            for (name, bag) in &mut self.bags {
                refill_one(name, bag);
            }
            for (name, table) in &mut self.tables {
                refill_one(name, table);
            }
            self.changed = true;
            return;
        }

        match self.resolve(name) {
            Some(Found::Bag(name, bag)) => refill_one(name, bag),
            Some(Found::Table(name, table)) => refill_one(name, table),
            None => return,
        }
        self.changed = true;
    }

    // `reset [name]`, or everything when no name is given. Unlike `refill`,
    // which puts every entry back, this goes back to the file: one that
    // starts with `remaining` or `used` set starts that way again.
    fn reset(&mut self, name: &str) {
        if name.is_empty() {
            self.bags = self.original.bags.clone().into_iter().collect();
            self.tables = self.original.tables.clone().into_iter().collect();
            self.changed = true;
            println!("everything is back to how its file has it");
            return;
        }

        // Taken apart so the lists can be borrowed while reading `original`.
        let Self {
            bags,
            tables,
            original,
            changed,
        } = self;

        match resolve(bags, tables, name) {
            Some(Found::Bag(name, bag)) => {
                let file = original.bags.get(name);
                *bag = file
                    .expect("internal error: every bag comes from the config")
                    .clone();
                println!("{name} is back to how its file has it");
            }
            Some(Found::Table(name, table)) => {
                let file = original.tables.get(name);
                *table = file
                    .expect("internal error: every table comes from the config")
                    .clone();
                println!("{name} is back to how its file has it");
            }
            None => return,
        }
        *changed = true;
    }

    fn resolve(&mut self, name: &str) -> Option<Found<'_>> {
        resolve(&mut self.bags, &mut self.tables, name)
    }
}

// Borrows the one bag or table `name` addresses, telling the user when that
// is not exactly one of them. Takes the lists rather than the session, so a
// caller can hold other parts of the session at the same time.
fn resolve<'s>(
    bags: &'s mut [NamedBag],
    tables: &'s mut [NamedTable],
    name: &str,
) -> Option<Found<'s>> {
    if name.is_empty() {
        println!("which table? `tables` lists them");
        return None;
    }

    let bag_matches = name_matches(bags, name);
    let table_matches = name_matches(tables, name);

    match (&bag_matches[..], &table_matches[..]) {
        ([index], []) => {
            let (name, bag) = &mut bags[*index];
            Some(Found::Bag(name, bag))
        }
        ([], [index]) => {
            let (name, table) = &mut tables[*index];
            Some(Found::Table(name, table))
        }
        ([], []) => {
            println!("no table called {name:?}, `tables` lists them");
            None
        }
        // Several entries share the prefix, or a bag and a table do.
        // Construction rejects duplicate names, so typing more resolves it.
        _ => {
            let bag_entries = bag_matches.iter().map(|index| &bags[*index]);
            let table_entries = table_matches.iter().map(|index| &tables[*index]);
            let matched: Vec<&str> = bag_entries
                .map(|(label, _)| label.as_str())
                .chain(table_entries.map(|(label, _)| label.as_str()))
                .collect();

            println!("{name:?} matches {}", matched.join(", "));
            None
        }
    }
}

// Indexes of the entries `name` addresses. An exact name wins outright, so a
// name that is also the prefix of another stays reachable.
fn name_matches<S>(entries: &[(String, S)], name: &str) -> Vec<usize> {
    if let Some(index) = entries.iter().position(|(label, _)| label == name) {
        return vec![index];
    }

    entries
        .iter()
        .enumerate()
        .filter_map(|(index, (label, _))| label.starts_with(name).then_some(index))
        .collect()
}

fn draw_from<S: Draw<Item = Encounter>>(name: &str, source: &mut S, count: usize) {
    for _ in 0..count {
        match source.draw() {
            Some(item) => println!("{}: {}", name.red(), item),
            None => {
                println!("{name} is empty, `refill {name}` to put it all back");
                break;
            }
        }
    }
}

// `"artifacts 3"` splits into `("artifacts", 3)`; anything without a trailing
// number keeps its whole text and draws once.
fn split_count(rest: &str) -> (&str, usize) {
    match rest.rsplit_once(' ') {
        Some((name, count)) => match count.parse() {
            Ok(count) => (name.trim(), count),
            Err(_) => (rest, 1),
        },
        None => (rest, 1),
    }
}

// One line of `tables`, as `  artifacts    100 left  unique`.
fn row<S: Draw>(name: &str, source: &S) -> String {
    let measure = if source.exhausts() { "left" } else { "rows" };
    let unique = if source.unique() { "  unique" } else { "" };

    format!("  {name:<12}{:>4} {measure}{unique}", source.remaining())
}

fn refill_one<S: Draw>(name: &str, source: &mut S) {
    source.fill();

    if source.exhausts() {
        println!("{name} refilled, {} available", source.remaining());
    } else {
        println!("{name} rolls fresh every time, nothing to put back");
    }
}

fn roll(expr: &str) {
    match DiceExpr::parse(expr) {
        Some(dice) => println!("{expr} = {}", dice.eval()),
        None => println!("{expr:?} is not a dice expression, try `roll 2d6+3`"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use super::*;

    fn bag() -> Bag<Encounter> {
        Bag::try_new(
            HashMap::from([(Encounter::markup("relic"), 1.0)]),
            None,
            false,
        )
        .unwrap()
    }

    fn table(unique: bool) -> Table<Encounter> {
        Table::try_new(
            HashMap::from([(1..=10, (Encounter::markup("a diver"), false))]),
            unique,
        )
        .unwrap()
    }

    // A session straight from its files: what it starts with is the original.
    fn fresh(bags: Vec<NamedBag>, tables: Vec<NamedTable>) -> Session {
        let original = Tables {
            bags: bags.iter().cloned().collect(),
            tables: tables.iter().cloned().collect(),
            hashes: Default::default(),
        };
        Session::new(bags, tables, original)
    }

    // One bag and two tables, so a table index can land past the end of the
    // bag list.
    fn session() -> Session {
        fresh(
            vec![("artifacts".to_string(), bag())],
            vec![
                ("encounters".to_string(), table(false)),
                ("rumours".to_string(), table(true)),
            ],
        )
    }

    // How many are left in the bag or table called `name`.
    fn left(session: &Session, name: &str) -> usize {
        let bag = session.bags.iter().find(|(label, _)| label == name);
        let table = session.tables.iter().find(|(label, _)| label == name);

        match (bag, table) {
            (Some((_, bag)), None) => bag.remaining(),
            (None, Some((_, table))) => table.remaining(),
            _ => panic!("no single entry called {name:?}"),
        }
    }

    fn resolved<'a>(session: &'a mut Session, name: &str) -> Option<&'a str> {
        match session.resolve(name)? {
            Found::Bag(name, _) | Found::Table(name, _) => Some(name),
        }
    }

    #[test]
    fn names_resolve_across_both_lists() {
        let mut session = session();

        assert_eq!(resolved(&mut session, "artifacts"), Some("artifacts"));
        assert_eq!(resolved(&mut session, "encounters"), Some("encounters"));
        assert_eq!(resolved(&mut session, "rumours"), Some("rumours"));
        assert_eq!(resolved(&mut session, "nope"), None);
    }

    #[test]
    fn prefixes_resolve_when_they_match_one_entry() {
        let mut session = session();

        assert_eq!(resolved(&mut session, "art"), Some("artifacts"));
        assert_eq!(resolved(&mut session, "enc"), Some("encounters"));
        assert_eq!(resolved(&mut session, "r"), Some("rumours"));
    }

    #[test]
    fn an_exact_name_wins_over_a_longer_one() {
        let mut session = fresh(
            vec![("art".to_string(), bag()), ("artifacts".to_string(), bag())],
            vec![],
        );

        assert_eq!(resolved(&mut session, "art"), Some("art"));
        assert_eq!(resolved(&mut session, "artif"), Some("artifacts"));
    }

    #[test]
    fn a_prefix_matching_a_bag_and_a_table_resolves_to_neither() {
        let mut session = fresh(
            vec![("relics".to_string(), bag())],
            vec![("rumours".to_string(), table(false))],
        );

        assert_eq!(resolved(&mut session, "r"), None);
    }

    #[test]
    #[should_panic(expected = "names have to be unique")]
    fn a_name_used_by_a_bag_and_a_table_is_rejected() {
        fresh(
            vec![("loot".to_string(), bag())],
            vec![("loot".to_string(), table(false))],
        );
    }

    #[test]
    fn a_table_past_the_end_of_the_bags_is_safe() {
        let mut session = session();

        session.command("draw rumours");
        assert_eq!(left(&session, "rumours"), 0, "a unique row is consumed");

        session.command("refill rumours");
        assert_eq!(left(&session, "rumours"), 1);
    }

    #[test]
    fn drawing_from_a_table_leaves_the_bags_alone() {
        let mut session = session();

        session.command("draw encounters 3");

        assert_eq!(left(&session, "artifacts"), 1);
    }

    #[test]
    fn refill_with_no_name_covers_both_kinds() {
        let mut session = session();
        session.command("draw artifacts");
        session.command("draw rumours");
        assert_eq!(left(&session, "artifacts"), 0);
        assert_eq!(left(&session, "rumours"), 0);

        session.command("refill");

        assert_eq!(left(&session, "artifacts"), 1);
        assert_eq!(left(&session, "rumours"), 1);
    }

    #[test]
    fn split_count_only_takes_a_trailing_number() {
        assert_eq!(split_count("artifacts 3"), ("artifacts", 3));
        assert_eq!(split_count("artifacts"), ("artifacts", 1));
        assert_eq!(
            split_count("the reactor is failing"),
            ("the reactor is failing", 1)
        );
    }

    #[test]
    fn a_row_counts_down_only_when_drawing_exhausts() {
        let session = session();
        let (name, bag) = &session.bags[0];
        let (rolling, table) = &session.tables[0];
        let (consuming, unique_table) = &session.tables[1];

        assert!(row(name, bag).contains("1 left"));
        assert!(row(rolling, table).contains("1 rows"));
        assert!(row(consuming, unique_table).contains("1 left  unique"));
    }

    fn saves(session: &mut Session, line: &str) -> usize {
        let mut count = 0;
        session.command(line);
        session.save_if_changed(&mut |_: &[NamedBag], _: &[NamedTable]| count += 1);
        count
    }

    #[test]
    fn drawing_and_refilling_save() {
        let mut session = session();

        assert_eq!(saves(&mut session, "draw artifacts"), 1);
        assert_eq!(saves(&mut session, "refill artifacts"), 1);
        assert_eq!(saves(&mut session, "refill"), 1);
    }

    #[test]
    fn looking_and_rolling_do_not_save() {
        let mut session = session();

        for line in [
            "tables",
            "help",
            "roll 2d6",
            "render *x*",
            "",
            "draw nothing-by-that-name",
        ] {
            assert_eq!(saves(&mut session, line), 0, "{line:?}");
        }
    }

    #[test]
    fn resetting_saves() {
        let mut session = session();

        assert_eq!(saves(&mut session, "reset art"), 1);
        assert_eq!(saves(&mut session, "reset"), 1);
        assert_eq!(saves(&mut session, "reset nothing-by-that-name"), 0);
    }

    // A bag whose file starts it part-drawn: 3 of 4 left.
    fn part_drawn() -> Bag<Encounter> {
        Bag::try_new(
            HashMap::from([
                (Encounter::markup("common"), 0.75),
                (Encounter::markup("rare"), 0.25),
            ]),
            Some(HashMap::from([
                (Encounter::markup("common"), 2),
                (Encounter::markup("rare"), 1),
            ])),
            false,
        )
        .unwrap()
    }

    #[test]
    fn reset_goes_back_to_the_file_where_refill_fills_up() {
        let mut session = fresh(vec![("loot".to_string(), part_drawn())], Vec::new());
        session.command("draw loot 3");
        assert_eq!(left(&session, "loot"), 0);

        session.command("refill loot");
        assert_eq!(left(&session, "loot"), 4);

        session.command("reset lo");
        assert_eq!(left(&session, "loot"), 3);
    }

    #[test]
    fn reset_with_no_name_undoes_a_restored_session() {
        // Restored from a save with the bag emptied, over a file that has it
        // part-drawn.
        let emptied = {
            let mut emptied = part_drawn();
            while emptied.draw().is_some() {}
            emptied
        };
        let original = Tables {
            bags: BTreeMap::from([("loot".to_string(), part_drawn())]),
            tables: BTreeMap::from([("rumours".to_string(), table(true))]),
            hashes: Default::default(),
        };
        let used = {
            let mut used = table(true);
            used.draw();
            used
        };
        let mut session = Session::new(
            vec![("loot".to_string(), emptied)],
            vec![("rumours".to_string(), used)],
            original,
        );

        session.command("reset");

        assert_eq!(left(&session, "loot"), 3);
        assert_eq!(left(&session, "rumours"), 1);
    }
}
