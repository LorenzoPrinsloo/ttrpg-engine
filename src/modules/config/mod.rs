/* Tables read from disk when the program starts. The directory says which
   kind a file is, and the file stem is the table's name:

   ```text
   tables/bags/artifacts.toml     -> a Bag called "artifacts"
   tables/tables/rumours.toml     -> a Table called "rumours"
   ```

   A bag — ratios are fractions of the whole bag and have to add up to 1.0,
   and `unique` means an entry is gone once it has come up:

   ```toml
   unique = true

   [[items]]
   ratio = 0.19
   text = '''**[1d10] Black Marble(s).** Undiscovered material. **10kcr each.**'''
   ```

   A table — inclusive ranges that must not overlap. Gaps are allowed, and a
   roll landing in one is rerolled:

   ```toml
   [[rows]]
   range = "01-19"
   text = '''Nothing but static.'''
   ```

   Entry text is markup, so write it in `'''` literal strings: markup owns
   `\` as its own escape, and a TOML basic string would eat it first.

   Top-level keys have to stay above the first `[[items]]` or `[[rows]]`.
   After that header they belong to the entry instead of the file. */

mod error;
mod file;
mod loader;
mod paths;
mod range;

pub use error::LoadError;
pub use loader::{Tables, load};
