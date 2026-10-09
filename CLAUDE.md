# ttrpg-engine

A terminal table roller for Mothership: weighted bags and ranged tables loaded
from TOML, drawn from an interactive shell or one-shot commands, with the state
of each game saved as a named session.

## Commands

```sh
cargo test                       # unit tests + tests/ (end-to-end, runs the binary)
cargo clippy --all-targets       # must be clean for code you touch
cargo fmt                        # the tree is rustfmt-clean; keep it that way
cargo run -- [--session NAME] [command...]
```

`MOTHERSHIP_TABLES` overrides the config directory, `MOTHERSHIP_SESSIONS` the
sessions directory (default: platform data dir + `ttrpg-engine/sessions`).

## Layout

- `src/modules/config/` – finds the config root, loads `bags/*.toml` and
  `tables/*.toml` through figment, fingerprints each file.
- `src/modules/bags/`, `src/modules/tables/` – `Bag` and `Table`, both `Draw`.
  They (de)serialize through `BagFile` / `RowsFile`, the same shape as the TOML,
  so a saved session and a config file share one format.
- `src/modules/persistence/` – `SessionFile` (what is saved, `capture`/`restore`)
  and `SessionStore` (atomic save, versioned load).
- `src/modules/shell.rs` – commands; autosaves through a callback after any
  command that changed state.
- `src/main.rs` – wires config → saved session → shell.

Behaviour that is easy to break:
- A saved bag/table is restored only if its file's fingerprint is unchanged;
  otherwise it starts as the file has it and a note goes to stderr.
- A session file that exists but will not load stops the program: carrying on
  would overwrite it on the first draw.
- One-shot commands print only their answer on stdout; notes go to stderr.

## Rust rules

These follow the [Canonical Rust best practices](https://canonical.github.io/rust-best-practices/),
the [Rust API guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)
(the parts that apply to a binary crate) and
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/). Where they
disagree, or disagree with established code here, the resolution is stated.
New code should also be locally consistent with the code around it.

### Gate

- `cargo test` passes and `cargo clippy --all-targets` adds no warnings.
- Don't put `#![deny(warnings)]` in the source. Deny specific lints by name if needed.

### Formatting

- `cargo fmt --check` must pass. The settings are rustfmt's defaults (stable toolchain, no
  `rustfmt.toml`).
- Use blank lines to separate meaning, not to look tidy. A declaration and the check on it stay together:
  `let x = foo();` directly followed by `if !x.is_valid() { … }`. Add a blank line after
  a declaration only when more than one later block uses it.
- Keep a closure next to the code that uses it. A closure that captures nothing should be an `fn`.
- Write hex in lowercase with `_` separators: `0xcbf2_9ce4_8422_2325`.

### Naming

- Follow RFC 430 casing. Don't abbreviate unless the abbreviation is standard,
  don't put types in names (`needle`, not `needle_str`), and use one name per concept.
- Use UK spelling (`colour`, `rumours`, `initialise`), matching the shipped tables.
- Getters have no `get_` prefix. Conversions are `as_`/`to_`/`into_`. Iterator methods are
  `iter`/`iter_mut`/`into_iter`.
- Pattern bindings reuse the field or variable name:
  `if let Some(response) = event.response`, `let Self { name, path } = self;`.
  Avoid `Some(r)` and avoid renaming in patterns.
- Generic type parameters are single letters. Lifetimes are named after the data they
  borrow (`'src`, `'line`), never numbered. Write `'_` rather than omitting it (`Found<'_>`).
- If no good name for something can be found, rethink the design rather than settle for a poor name.

### Imports and modules

- No glob imports, apart from `use super::*` in a `#[cfg(test)]` module.
  Don't glob-import enum variants. Alias a long enum instead (`use SomeEnum as Se;`).
- Group imports in three blocks separated by blank lines: `std` first, then third-party
  crates, then `crate`/`super`/`self`. Stable rustfmt sorts within a block but
  won't split the blocks, so place them by hand. Merge each path up to its last level and
  brace only that level: `use std::io::{self, BufRead, Write};`. Don't nest braces.
- `mod.rs` only declares structure, in this order: `pub mod`, `pub(crate) mod`, `mod`,
  `pub use`, `pub(crate) use`, `use`. Put `#[cfg]` items last.
  New multi-file modules use `foo/mod.rs`.
- Default to `pub(crate)`. Make something `pub` only when `main.rs` or
  `tests/` needs it.

### Types and API shape

- Constructors are associated functions returning `Self`. Data a user edits by
  hand (TOML, session files) goes through a fallible `try_new` that returns a
  typed error, never a panic.
- Implement the common traits eagerly where they make sense (`Clone`, `Debug`, `Default`,
  `PartialEq`, `Eq`, `Hash`). Every type should have a non-empty `Debug`.
- Use `From`/`TryFrom` for conversions, as `Bag` ⇄ `BagFile` already does,
  rather than ad-hoc functions.
- Put a single `#[derive]` on each type: `Copy` first, then std traits in alphabetical order, then
  third-party traits in alphabetical order: `#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]`.
- Use newtypes to keep apart values that share a representation. Prefer an enum
  over a `bool` or `Option` parameter whose meaning isn't clear at the call site (C-CUSTOM-TYPE).
- Keep struct fields private unless the struct is plain data. Order fields `pub`, then
  `pub(crate)`, then private.
- Take borrowed types as arguments: `&str`, `&[T]`, `&Path`, not `&String`/`&Vec<T>`/`&PathBuf`.
  Let the caller decide where data is copied (C-CALLER-CONTROL).
- No out-parameters. If a function has an obvious receiver, make it a method.
- Builders only when a type really needs one (YAGNI). When one is needed: `Foo::builder()` returns
  `FooBuilder`, its methods take `self` by value, and `.build()` is fallible.
  `FooBuilder` has no public constructor.
- Don't use `Deref` to fake inheritance. Use `mem::take`/`mem::replace` to move
  out of a `&mut` rather than cloning.

### Code

- Use `Self` in signatures and for constructing `Self`. Build other types by their
  concrete name.
- Struct literals: use field-init shorthand from same-named `let`s, and fill fields in
  declaration order. If some fields need computing, compute them all in `let`s
  first.
- Don't leave `let x;` uninitialised. Use `let x = if … { } else { };` or a `loop` with `break value`.
- Keep `let mut` in a narrow scope:
  `let data = { let mut data = load(); data.sort(); data };`.
  Prefer iterator chains to counter loops.
- Shadowing: at most one level in nested scopes. A type-changing shadow at most
  once per variable. Never shadow a type name.
- Annotate types only where the compiler needs it, in this order of preference:
  `let v: Vec<_> = …`, then turbofish, then fully qualified syntax.
- Put all generic bounds in one place. Use `where` if any bound needs it, or if the
  angle brackets get longer than about 30 characters. Use `impl Trait` for a single-use parameter.
- Don't call a method on an expression ending in `}`. Bind it to a name first.
- Format strings use inline arguments: `format!("{path}: {error}")`.
- Use `.collect()`, not `Vec::from_iter`. Use `Vec::new()` or `Vec::with_capacity(n)`,
  not `vec![]`, for an empty vector.
- Avoid `.0`/`.1` on tuples; destructure them. This is fine inside a newtype's own `impl`.
- Don't pattern-match in `fn` parameters (closures are exempt). Unpack on the first line of the body.
  Inside the crate, destructure exhaustively (`let Self { a, b, c: _ } = self;`) where
  a new field should force a revisit.
- End `()`-returning statements with `;`. Empty match arms are `=> {}`. Use a tail expression
  rather than `return` at the end of a function (clippy `needless_return`).
- Prefer a scope to `drop(x)`.
- Don't add `.clone()` just to satisfy the borrow checker. A clone should be deliberate and
  cheap enough not to matter.

### Errors and panics

- Each module has its own error enum (`LoadError`, `StoreError`, `BagError`,
  `TableError`) with hand-written `Display` and `std::error::Error`. There is no
  `thiserror`. Don't add it, or any other dependency, without asking.
- Error messages:
  - Write them for the person running the program.
  - Start lowercase.
  - Start with the file's path when a file is involved (`{path}: …`). Otherwise prefer
    the "cannot …" form.
  - Say what to do next when the user can fix it ("set MOTHERSHIP_SESSIONS…").
  - Don't repeat context that an outer error already adds.
- A wrapped error goes in a field named `source` in new variants. Older variants
  use `error`; rename them only when touching that code anyway.
- Convert a foreign error at the call that produces it (`.map_err` right there).
  Don't box errors (`Box<dyn Error>`) outside throwaway code.
- Panic only when the program itself is at fault, never because of user input or file
  content. Prefer making the bad state impossible through the types. Failing that, use `.expect("…")`
  stating the precondition, and start the message for an internal fault with
  `internal error:`. Use `.unwrap()` only where the reason is obvious in a line or two.

### Comments

- Keep the house style: `//` line comments that explain **why**, written in
  plain sentences, above the item or line they explain (a short trailing
  `// …` is fine). Don't use `/* … */`: rustfmt strips the indent inside it. Don't add `///` doc
  comments or comments that restate the code.
- Refer to parameters by name and say "the", not "a": "the given `name`".

### Tests

- Put unit tests in a `#[cfg(test)] mod tests` at the bottom of the file. Put end-to-end
  tests, which run the real binary, in `tests/`.
- Name a test as a sentence saying what it checks:
  `a_depleted_bag_stays_depleted`, `an_edited_file_starts_full_and_says_so`.
- Use `.unwrap()` in tests, not `?`, so a failure points at the line where it happened. Add the
  value to assertion messages (`assert!(…, "{error}")`).
- Never touch the real config or sessions directories. Use `tempfile::tempdir()`, and
  set `MOTHERSHIP_TABLES`/`MOTHERSHIP_SESSIONS` for the binary.
- Assert on parsed markup or raw text, not on rendered output: `colored`'s
  global switch makes rendering depend on which other tests are running at the same time.

### Known deviations in existing code

Fix these in code you are already changing. Don't sweep them up in an unrelated change:
- `src/modules.rs` sits beside `src/modules/` (the guides prefer `mod.rs`).
- Clippy: `result_large_err` from `figment::Error` in `LoadError`, plus a few
  minor lints.
- `Bag::try_new` / `Table::try_new` take a `unique: bool`. An enum would read
  better at call sites.
- `shell::Session::new` panics on duplicate names, which come from the config
  (user input). This should be a `LoadError`.
- `StoreError::Io`/`Json` name their inner error `error` rather than `source`.
