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

## Engine

- [ ] **Seedable randomness.** Every draw uses `rand::rng()`, so a session can't be replayed and tests can only check statistics, never exact sequences. A seed saved with the session would allow both.
- [ ] **Unique table draws by rejection.** A draw from a nearly used-up unique table rerolls until it hits an available row. Choosing among the available rows, weighted by width, would take one roll.

## Known deviations from CLAUDE.md

- [ ] `Bag::try_new` and `Table::try_new` take `unique: bool`; an enum would read better at call sites.
- [ ] `shell::Session::new` panics on duplicate names, which come from the config. This should be a `LoadError`.
- [ ] `StoreError::Io` and `StoreError::Json` name their inner error `error` rather than `source`.
- [ ] Clippy: `result_large_err` from `figment::Error` in `LoadError`, a `to_vec` suggestion in `Table::find_overlap`, and two needless borrows in `config/file.rs`.
- [ ] `src/modules.rs` sits beside `src/modules/` instead of being `src/modules/mod.rs`.
