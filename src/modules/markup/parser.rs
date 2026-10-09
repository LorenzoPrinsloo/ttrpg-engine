use std::str::FromStr;

use colored::Color;

use crate::markup::{DiceExpr, Node};

// The style markers. Each one both opens and closes its span.
const BOLD: &str = "**";
const ITALIC: &str = "*";
const UNDERLINE: &str = "__";

// Spans nested deeper than this are treated as malformed, keeping the
// recursion below off a stack that pathological input could exhaust. Real
// text never comes close.
const MAX_DEPTH: usize = 64;

// Parses markup into a tree of [`Node`]s. This never fails: malformed
// markup comes back as literal text.
pub fn parse(input: &str) -> Vec<Node> {
    // With no closing marker to look for, parsing can only stop at the end
    // of the input, so this cannot be `Err`.
    Parser::new(input).children(None).unwrap_or_default()
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    depth: usize,
}

// The expected closing marker was never found before the end of the input.
struct Unclosed;

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            depth: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn starts_with(&self, marker: &str) -> bool {
        marker
            .chars()
            .enumerate()
            .all(|(i, c)| self.chars.get(self.pos + i) == Some(&c))
    }

    // Consumes `marker` when it closes the span being parsed. A `**` inside
    // an italic span opens bold rather than closing the italic, even though
    // the italic marker is a prefix of it.
    fn eat_closing(&mut self, marker: &str) -> bool {
        if !self.starts_with(marker) || (marker == ITALIC && self.starts_with(BOLD)) {
            return false;
        }
        self.pos += marker.chars().count();

        true
    }

    // Parses nodes until `closing` is found (and consumed) or, when
    // `closing` is `None`, until the end of the input.
    //
    // This is the only recursive function in the grammar: every span
    // re-enters it with its own closing marker.
    fn children(&mut self, closing: Option<&str>) -> Result<Vec<Node>, Unclosed> {
        let mut nodes = Vec::new();

        loop {
            // Looking for the closing marker before any opening marker is
            // what stops a marker nesting inside itself: inside bold, `**`
            // always closes.
            if closing.is_some_and(|marker| self.eat_closing(marker)) {
                return Ok(nodes);
            }
            let Some(c) = self.peek() else {
                return if closing.is_some() {
                    Err(Unclosed)
                } else {
                    Ok(nodes)
                };
            };

            let start = self.pos;
            let parsed = match c {
                '\\' => {
                    self.bump();
                    // A trailing lone backslash is literal.
                    push_char(&mut nodes, self.bump().unwrap_or('\\'));
                    continue;
                }
                '*' if self.starts_with(BOLD) => self.span(BOLD).map(Node::Bold),
                '*' => self.span(ITALIC).map(Node::Italic),
                '_' if self.starts_with(UNDERLINE) => self.span(UNDERLINE).map(Node::Underline),
                '{' => self.color(),
                '[' => self.dice(),
                _ => {
                    self.pos += 1;
                    push_char(&mut nodes, c);
                    continue;
                }
            };

            match parsed {
                Some(node) => nodes.push(node),
                // Malformed: rewind, emit the opening character literally
                // and let the loop rescan from just after it. That single
                // character step is what turns `**never closed` into text.
                None => {
                    self.pos = start + 1;
                    push_char(&mut nodes, c);
                }
            }
        }
    }

    // Parses the body of a span, refusing to nest past [`MAX_DEPTH`].
    // Returning `None` hands the span to the malformed path, which is the
    // same place unclosed markers end up.
    fn nested(&mut self, closing: &str) -> Option<Vec<Node>> {
        if self.depth == MAX_DEPTH {
            return None;
        }
        self.depth += 1;
        let children = self.children(Some(closing)).ok();
        self.depth -= 1;

        children
    }

    // `**...**` or `__...__`.
    fn span(&mut self, marker: &str) -> Option<Vec<Node>> {
        self.pos += marker.chars().count();
        self.nested(marker)
    }

    // `{colour|text}`. The colour is everything up to the *first* `|`; any
    // later pipes are part of the text.
    fn color(&mut self) -> Option<Node> {
        self.pos += 1;

        let mut name = String::new();
        loop {
            match self.bump()? {
                '|' => break,
                '{' | '}' => return None,
                c => name.push(c),
            }
        }

        let color = parse_color(name.trim())?;
        let children = self.nested("}")?;

        Some(Node::Color { color, children })
    }

    // `[2d6+3]`. Expression context: the content is read as raw characters,
    // with no markup inside, and then has to parse as a dice expression.
    fn dice(&mut self) -> Option<Node> {
        self.pos += 1;

        let mut raw = String::new();
        loop {
            match self.bump()? {
                ']' => break,
                c => raw.push(c),
            }
        }

        DiceExpr::parse(&raw).map(Node::Dice)
    }
}

fn push_char(nodes: &mut Vec<Node>, c: char) {
    match nodes.last_mut() {
        Some(Node::Text(text)) => text.push(c),
        _ => nodes.push(Node::Text(c.to_string())),
    }
}

fn parse_color(name: &str) -> Option<Color> {
    let name = name.to_ascii_lowercase();

    if let Some(hex) = name.strip_prefix('#') {
        return parse_hex(hex);
    }

    match name.as_str() {
        "gray" | "grey" => Some(Color::BrightBlack),
        "orange" => Some(Color::TrueColor {
            r: 255,
            g: 165,
            b: 0,
        }),
        "pink" => Some(Color::TrueColor {
            r: 255,
            g: 192,
            b: 203,
        }),
        "brown" => Some(Color::TrueColor {
            r: 165,
            g: 42,
            b: 42,
        }),
        _ => Color::from_str(&name).ok(),
    }
}

fn parse_hex(hex: &str) -> Option<Color> {
    let digits: Vec<u8> = hex
        .chars()
        .map(|c| c.to_digit(16).map(|d| d as u8))
        .collect::<Option<_>>()?;

    let [r, g, b] = match digits[..] {
        [r, g, b] => [r * 17, g * 17, b * 17], // #rgb widens to #rrggbb
        [r1, r2, g1, g2, b1, b2] => [r1 * 16 + r2, g1 * 16 + g2, b1 * 16 + b2],
        _ => return None,
    };

    Some(Color::TrueColor { r, g, b })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Node {
        Node::Text(s.to_string())
    }

    fn dice(raw: &str) -> Node {
        Node::Dice(DiceExpr::parse(raw).expect("valid dice expression"))
    }

    fn colored(color: Color, children: Vec<Node>) -> Node {
        Node::Color { color, children }
    }

    #[test]
    fn plain_text_is_one_node() {
        assert_eq!(parse("hello world"), vec![text("hello world")]);
    }

    #[test]
    fn bold_and_underline() {
        assert_eq!(
            parse("a **b** c __d__"),
            vec![
                text("a "),
                Node::Bold(vec![text("b")]),
                text(" c "),
                Node::Underline(vec![text("d")]),
            ]
        );
    }

    #[test]
    fn italic() {
        assert_eq!(
            parse("Organics must *Sanity Save*"),
            vec![
                text("Organics must "),
                Node::Italic(vec![text("Sanity Save")])
            ]
        );
    }

    #[test]
    fn bold_wins_over_italic() {
        // `**` opens bold inside an italic span instead of closing it.
        assert_eq!(
            parse("*a **b** c*"),
            vec![Node::Italic(vec![
                text("a "),
                Node::Bold(vec![text("b")]),
                text(" c"),
            ])]
        );
        assert_eq!(
            parse("**a *b* c**"),
            vec![Node::Bold(vec![
                text("a "),
                Node::Italic(vec![text("b")]),
                text(" c"),
            ])]
        );
        // Which leaves `***x***` ambiguous: the `**` pair binds and the odd
        // asterisks stay literal.
        assert_eq!(
            parse("***x***"),
            vec![text("*"), Node::Bold(vec![text("x")]), text("*")]
        );
    }

    #[test]
    fn an_unpaired_asterisk_is_text() {
        assert_eq!(parse("2 * 3 = 6"), vec![text("2 * 3 = 6")]);
    }

    #[test]
    fn a_single_marker_character_is_text() {
        assert_eq!(parse("2 * 3 and a_b_c"), vec![text("2 * 3 and a_b_c")]);
    }

    #[test]
    fn styles_compose_in_any_order() {
        assert_eq!(
            parse("**__{red|x}__**"),
            vec![Node::Bold(vec![Node::Underline(vec![colored(
                Color::Red,
                vec![text("x")]
            )])])]
        );
        assert_eq!(
            parse("{red|**__x__**}"),
            vec![colored(
                Color::Red,
                vec![Node::Bold(vec![Node::Underline(vec![text("x")])])]
            )]
        );
    }

    #[test]
    fn styling_can_be_partial() {
        assert_eq!(
            parse("{red|plain **bold** plain}"),
            vec![colored(
                Color::Red,
                vec![
                    text("plain "),
                    Node::Bold(vec![text("bold")]),
                    text(" plain")
                ]
            )]
        );
    }

    #[test]
    fn same_marker_closes_instead_of_nesting() {
        // The second `**` closes the first span, and the third opens a new
        // one that the fourth closes - the same way Markdown behaves.
        assert_eq!(
            parse("**a **b** c**"),
            vec![
                Node::Bold(vec![text("a ")]),
                text("b"),
                Node::Bold(vec![text(" c")]),
            ]
        );
    }

    #[test]
    fn first_pipe_splits_the_colour() {
        assert_eq!(
            parse("{red|either|or}"),
            vec![colored(Color::Red, vec![text("either|or")])]
        );
    }

    #[test]
    fn colour_names_are_case_insensitive_and_trimmed() {
        assert_eq!(
            parse("{ Bright Cyan |x}"),
            vec![colored(Color::BrightCyan, vec![text("x")])]
        );
    }

    #[test]
    fn extra_colour_names() {
        assert_eq!(
            parse("{grey|x}"),
            vec![colored(Color::BrightBlack, vec![text("x")])]
        );
        assert_eq!(
            parse("{orange|x}"),
            vec![colored(
                Color::TrueColor {
                    r: 255,
                    g: 165,
                    b: 0
                },
                vec![text("x")]
            )]
        );
    }

    #[test]
    fn hex_colours() {
        let orange = Color::TrueColor {
            r: 255,
            g: 136,
            b: 0,
        };
        assert_eq!(parse("{#ff8800|x}"), vec![colored(orange, vec![text("x")])]);
        // Three digit hex widens each digit: #f80 == #ff8800.
        assert_eq!(parse("{#F80|x}"), vec![colored(orange, vec![text("x")])]);
    }

    #[test]
    fn malformed_colours_are_literal() {
        assert_eq!(parse("{notacolour|text}"), vec![text("{notacolour|text}")]);
        assert_eq!(parse("{#ff88|x}"), vec![text("{#ff88|x}")]);
        assert_eq!(parse("{red no pipe}"), vec![text("{red no pipe}")]);
        assert_eq!(parse("{}"), vec![text("{}")]);
        // A non-ASCII colour name must not panic while being rejected.
        assert_eq!(parse("{#aáxyz|x}"), vec![text("{#aáxyz|x}")]);
    }

    #[test]
    fn unclosed_spans_are_literal() {
        assert_eq!(parse("**never closed"), vec![text("**never closed")]);
        assert_eq!(parse("__nor this"), vec![text("__nor this")]);
        assert_eq!(parse("{red|no brace"), vec![text("{red|no brace")]);
        assert_eq!(parse("[1d6 no bracket"), vec![text("[1d6 no bracket")]);
    }

    #[test]
    fn spans_cannot_overlap() {
        // `__` cannot close while bold is still open, so the bold span never
        // finds its end and the whole thing degrades to text - except for
        // the inner `__b **c__` pair, which does nest properly.
        assert_eq!(
            parse("**a __b **c__"),
            vec![text("**a "), Node::Underline(vec![text("b **c")])]
        );
    }

    #[test]
    fn dice_expressions() {
        assert_eq!(parse("[2d6+3]"), vec![dice("2d6+3")]);
        assert_eq!(
            parse("You take [2d6] DMG"),
            vec![text("You take "), dice("2d6"), text(" DMG")]
        );
    }

    #[test]
    fn no_markup_inside_an_expression() {
        // `**` is part of the expression text, which then fails to parse, so
        // the `[` degrades to literal text. From there the content is
        // ordinary text again - and ordinary text is markup context, so the
        // `**6**` inside it does become bold.
        assert_eq!(
            parse("[2d**6**]"),
            vec![text("[2d"), Node::Bold(vec![text("6")]), text("]")]
        );
    }

    #[test]
    fn malformed_dice_is_literal() {
        assert_eq!(parse("[see pg. 12]"), vec![text("[see pg. 12]")]);
        assert_eq!(parse("Sanity [+] Save"), vec![text("Sanity [+] Save")]);
        assert_eq!(parse("[0d6]"), vec![text("[0d6]")]);
    }

    #[test]
    fn escapes() {
        assert_eq!(parse("\\*\\*x\\*\\*"), vec![text("**x**")]);
        assert_eq!(parse("\\[1d6]"), vec![text("[1d6]")]);
        assert_eq!(parse("\\{red|x}"), vec![text("{red|x}")]);
        assert_eq!(parse("a\\\\b"), vec![text("a\\b")]);
        assert_eq!(parse("trailing \\"), vec![text("trailing \\")]);
    }

    #[test]
    fn an_escaped_marker_does_not_close_a_span() {
        assert_eq!(
            parse("**a \\*\\* b**"),
            vec![Node::Bold(vec![text("a ** b")])]
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(parse(""), vec![]);
    }

    #[test]
    fn nesting_past_the_cap_is_malformed_rather_than_a_stack_overflow() {
        let deep = format!("{}x{}", "{red|".repeat(10_000), "}".repeat(10_000));
        // The spans within the cap still parse; everything past it is text.
        assert!(matches!(parse(&deep).first(), Some(Node::Color { .. })));
    }
}
