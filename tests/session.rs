/* The binary end to end: a draw in one run is still gone in the next, and an
   edit to the table file puts it back. */
use std::path::Path;
use std::process::{Command, Output};

const PAIR: &str = "unique = true\n\
    [[items]]\nratio = 0.5\ntext = 'left'\n\
    [[items]]\nratio = 0.5\ntext = 'right'\n";

fn run(config: &Path, sessions: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_ttrpg-engine"))
        .args(args)
        .env("MOTHERSHIP_TABLES", config)
        .env("MOTHERSHIP_SESSIONS", sessions)
        .output()
        .unwrap();

    assert!(output.status.success(), "{args:?}: {}", String::from_utf8_lossy(&output.stderr));
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn left(config: &Path, sessions: &Path, args: &[&str]) -> String {
    let listing = stdout(&run(config, sessions, args));
    listing.lines().find(|line| line.contains("pair")).unwrap().trim().to_string()
}

fn setup() -> (tempfile::TempDir, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("bags")).unwrap();
    std::fs::write(config.path().join("bags/pair.toml"), PAIR).unwrap();

    (config, tempfile::tempdir().unwrap())
}

#[test]
fn a_draw_survives_a_restart() {
    let (config, sessions) = setup();

    run(config.path(), sessions.path(), &["draw", "pair"]);

    assert!(sessions.path().join("default.json").is_file());
    assert_eq!(left(config.path(), sessions.path(), &["tables"]), "pair           1 left  unique");
}

#[test]
fn sessions_are_kept_apart_by_name() {
    let (config, sessions) = setup();

    run(config.path(), sessions.path(), &["--session", "derelict", "draw", "pair"]);

    assert_eq!(left(config.path(), sessions.path(), &["--session", "derelict", "tables"]), "pair           1 left  unique");
    assert_eq!(left(config.path(), sessions.path(), &["--session", "outpost", "tables"]), "pair           2 left  unique");
}

#[test]
fn editing_the_file_starts_it_full_with_a_note() {
    let (config, sessions) = setup();
    run(config.path(), sessions.path(), &["draw", "pair"]);

    std::fs::write(config.path().join("bags/pair.toml"), PAIR.replace("right", "wrong")).unwrap();
    let output = run(config.path(), sessions.path(), &["tables"]);

    assert!(stdout(&output).contains("pair           2 left"), "{}", stdout(&output));
    assert!(String::from_utf8_lossy(&output.stderr).contains("bags/pair.toml changed"));
}

#[test]
fn looking_saves_nothing() {
    let (config, sessions) = setup();

    run(config.path(), sessions.path(), &["tables"]);

    assert!(std::fs::read_dir(sessions.path()).unwrap().next().is_none());
}
