/* Inline markup for encounter and artifact text, rendered to ANSI for the
   terminal.

   ```text
   **text**        bold
   *text*          italic
   __text__        underline
   {color|text}    coloured text; the first `|` splits colour from text
   [2d6+3]         dice expression, rolled at render time
   \x              escape: the next character is literal
   ```

   Styles compose in any order and nest freely, but spans may never overlap
   and a marker cannot nest inside itself: in `**a **b** c**` the second
   `**` closes the first span. `[...]` is expression context, so no markup
   is parsed inside it, and dice results inherit the styles around them.

   Bold wins over italic wherever both could match, so `**a *b* c**` and
   `*a **b** c*` both nest, while `***x***` binds the `**` pair and leaves
   the odd asterisks literal.

   Anything malformed - an unclosed span, an unknown colour, a bad dice
   expression - renders as literal text. */

mod expr;
mod node;
mod parser;
mod render;

pub use expr::{DiceExpr, Sign, Term};
pub use node::Node;
pub use parser::parse;
pub use render::render;
pub use render::render_nodes;
