# TODO

What's next, from the [Kinds of random](README.md#kinds-of-random) design notes and from the code.

## Kinds of random

- [ ] **Encounter checks.** A `check` command that rolls d100 and treats doubles as an encounter (10%), then draws from the encounter bag. A `noise` command adds an extra check after gunfire or other loud noise.
- [ ] **Choose stateless or stateful per check.** Alongside d100 doubles, offer a shuffle bag of checks (for example one encounter token in ten) for games where the encounter rate is load-bearing.
- [ ] **Pity timer.** A draw type whose odds climb after every miss and reset on a hit, with a configurable starting chance and step.
- [ ] **Escalating severity.** The hybrid from the design notes: keep the encounter roll stateless, but let each quiet check add a token that makes the *next* encounter worse rather than likelier.
- [ ] **Ratchets.** Threshold tracks (The Bends, clocks, radiation) where exposure accumulates and effects switch on at fixed points, saved with the session.
- [ ] **Pressure vessel.** A stress track per character with panic checks: 2d10 at or under stress panics, the roll picks the result from the panic table, and severe results add stress. Stress carries over between sessions.
- [ ] **Pólya-urn bags.** A replacement rule that puts a drawn entry back with an extra copy, for pools that trend instead of settling.
- [ ] **`odds` command.** Each entry's chance on the next draw, given what's left in the bag. That's the "due" effect, made visible.
- [ ] **`simulate` command.** Draw from a bag, table or check N times and print the distribution, like the README charts but in the terminal.
- [ ] **Integer weights in bag files.** Ratios are rounded to hundredths, so a third can't be written exactly. Weights such as `1, 2, 2` would be exact.

## Modules and packaging

- [ ] **Loadable external modules.** A command such as `load path/to/module.toml` (or a directory of bags and tables) adds an external module to the current session, so people can write, share and mix modules for their games.
  - The session remembers which modules it loaded, with their paths and fingerprints, and loads them again on the next start, using the same edited-file rules as the built-in config.
  - Decide what happens when a module's bag or table has the same name as one already loaded: refuse, or namespace it (for example `derelict/encounters`).
  - An `unload` command, and listing modules in `tables`.
- [ ] **Package as an installable CLI.** Build standalone binaries for macOS, Linux and Windows, and publish them so people can install without a Rust toolchain: `cargo install`, GitHub releases and possibly Homebrew. Then go through the [Command Line Applications in Rust](https://rust-cli.github.io/book) book and bring the tool in line with its best practices:
  - argument parsing with `--help` and `--version`
  - meaningful exit codes, with errors on stderr and results on stdout
  - human-friendly output in a terminal, and plain output when piped (colour off unless forced)
  - handling ctrl-C cleanly mid-command
  - config and data directories that follow each platform's conventions, and finding the config when installed outside the repository
  - end-to-end tests of the installed binary
  - shell completions and a man page

  New dependencies such as `clap` need agreeing first, per CLAUDE.md.

## Look and feel

- [ ] **A title screen.** An ASCII-art banner when the shell starts, in the style of Mothership's retro ship computers, with the session name and what's loaded.
- [ ] **A consistent colour theme.** Colour by meaning: draws, notes, warnings, errors and dice results each look the same everywhere, building on the markup renderer's colours.
- [ ] **Richer output.** `tables` as an aligned, boxed table. Gauges for how full each bag is (`encounters ███████░░░ 7/10`). Rolls that show each die (`2d6+3 = [4, 2] + 3 = 9`).
- [ ] **A full-screen TUI mode.** Panes for the bags and tables, a log of what was drawn this session, and a command line, all keyboard-driven (for example with `ratatui`). Bag gauges and the `odds` view would update live as things are drawn.
- [ ] **Stay script-friendly.** All of this only in an interactive terminal: plain output when piped, and colour off with `NO_COLOR` (see the CLI book item above). New dependencies such as `ratatui` need agreeing first, per CLAUDE.md.

## Engine

- [ ] **Seedable randomness.** Every draw uses `rand::rng()`, so a session can't be replayed and tests can only check statistics, never exact sequences. A seed saved with the session would allow both.
- [ ] **Unique table draws by rejection.** A draw from a nearly used-up unique table rerolls until it hits an available row. Choosing among the available rows, weighted by width, would take one roll.

## Known deviations from CLAUDE.md

- [ ] `Bag::try_new` and `Table::try_new` take `unique: bool`; an enum would read better at call sites.
- [ ] `shell::Session::new` panics on duplicate names, which come from the config. This should be a `LoadError`.
- [ ] `StoreError::Io` and `StoreError::Json` name their inner error `error` rather than `source`.
- [ ] Clippy: `result_large_err` from `figment::Error` in `LoadError`, a `to_vec` suggestion in `Table::find_overlap`, and two needless borrows in `config/file.rs`.
- [ ] `src/modules.rs` sits beside `src/modules/` instead of being `src/modules/mod.rs`.
