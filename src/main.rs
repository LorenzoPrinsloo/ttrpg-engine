pub mod modules;

pub use modules::{bags, config, encounters, markup, persistence, shell};

use persistence::{SessionFile, SessionStore};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (name, args) = match split_session(args) {
        Ok(split) => split,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let (root, mut loaded) = match config::load() {
        Ok(loaded) => loaded,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            std::process::exit(1);
        }
    };

    /* A session that is there but will not load stops the start: carrying on
       with full tables would overwrite it on the first draw. */
    let store = SessionStore::locate().unwrap_or_else(|error| fail(error));
    match store.load(&name) {
        Ok(Some(saved)) => {
            for note in saved.restore(&mut loaded) {
                eprintln!("{note}");
            }
        }
        Ok(None) => {}
        Err(error) => fail(error),
    }

    /* Only when there is a session to have: a one-shot command should print
       nothing but its answer. */
    if args.is_empty() {
        println!(
            "{} bags and {} tables from {}, session {name:?}",
            loaded.bags.len(),
            loaded.tables.len(),
            root.display()
        );
    }

    let hashes = loaded.hashes;
    shell::run(
        loaded.bags.into_iter().collect(),
        loaded.tables.into_iter().collect(),
        &args,
        |bags, tables| {
            /* Reported but not fatal: the game in front of the user is still
               fine, and the next save may well succeed. */
            if let Err(error) = store.save(&name, &SessionFile::capture(bags, tables, &hashes)) {
                eprintln!("not saved: {error}");
            }
        },
    );
}

fn fail(error: impl std::fmt::Display) -> ! {
    eprintln!("{error}");
    std::process::exit(1);
}

/* `--session NAME` (or `--session=NAME`) picks the session, `default` when
   absent. Only read ahead of the command, so a command's own text can never
   be taken for it. */
fn split_session(mut args: Vec<String>) -> Result<(String, Vec<String>), String> {
    const FLAG: &str = "--session";

    let name = match args.first().map(String::as_str) {
        Some(FLAG) => match args.get(1) {
            Some(name) => {
                let name = name.clone();
                args.drain(..2);
                name
            }
            None => return Err(format!("{FLAG} needs a name")),
        },
        Some(first) if first.starts_with("--session=") => {
            let name = first[FLAG.len() + 1..].to_string();
            args.remove(0);
            name
        }
        _ => "default".to_string(),
    };

    Ok((name, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(args: &[&str]) -> Result<(String, Vec<String>), String> {
        split_session(args.iter().map(|arg| arg.to_string()).collect())
    }

    #[test]
    fn without_the_flag_the_session_is_default() {
        assert_eq!(split(&["draw", "artifacts"]).unwrap(), ("default".to_string(), vec!["draw".to_string(), "artifacts".to_string()]));
    }

    #[test]
    fn the_flag_takes_the_next_argument() {
        assert_eq!(split(&["--session", "derelict", "draw", "artifacts"]).unwrap().0, "derelict");
        assert_eq!(split(&["--session=derelict"]).unwrap(), ("derelict".to_string(), vec![]));
    }

    #[test]
    fn the_flag_is_not_read_out_of_a_command() {
        let (name, args) = split(&["render", "--session", "x"]).unwrap();

        assert_eq!(name, "default");
        assert_eq!(args, ["render", "--session", "x"]);
    }

    #[test]
    fn the_flag_without_a_name_is_an_error() {
        assert!(split(&["--session"]).unwrap_err().contains("needs a name"));
    }
}
