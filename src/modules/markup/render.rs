use crate::markup::{Node, parse};
use colored::{Color, ColoredString, Colorize};

pub fn render(input: &str) -> String {
    render_nodes(&parse(input))
}

pub fn render_nodes(nodes: &[Node]) -> String {
    let mut out = String::new();
    write_nodes(nodes, Style::default(), &mut out);
    out
}

#[derive(Debug, Clone, Copy, Default)]
struct Style {
    bold: bool,
    italic: bool,
    underline: bool,
    color: Option<Color>,
}

impl Style {
    fn apply(self, text: &str) -> ColoredString {
        let mut styled = text.normal();
        if self.bold {
            styled = styled.bold();
        }
        if self.italic {
            styled = styled.italic();
        }
        if self.underline {
            styled = styled.underline();
        }
        if let Some(color) = self.color {
            styled = styled.color(color);
        }
        styled
    }
}

fn write_nodes(nodes: &[Node], style: Style, out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(text) => out.push_str(&style.apply(text).to_string()),
            Node::Dice(expr) => out.push_str(&style.apply(&expr.eval().to_string()).to_string()),
            Node::Bold(children) => write_nodes(children, Style { bold: true, ..style }, out),
            Node::Italic(children) => write_nodes(children, Style { italic: true, ..style }, out),
            Node::Underline(children) => {
                write_nodes(children, Style { underline: true, ..style }, out)
            }
            Node::Color { color, children } => {
                write_nodes(children, Style { color: Some(*color), ..style }, out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* Tests assert against ANSI output, so colouring must not be turned off
       by the environment `cargo test` happens to run in. */
    fn render_colored(input: &str) -> String {
        colored::control::set_override(true);
        render(input)
    }

    #[test]
    fn plain_text_is_left_alone() {
        assert_eq!(render_colored("just text"), "just text");
    }

    #[test]
    fn single_styles() {
        assert_eq!(render_colored("**b**"), "b".bold().to_string());
        assert_eq!(render_colored("__u__"), "u".underline().to_string());
        assert_eq!(render_colored("{red|r}"), "r".red().to_string());
    }

    #[test]
    fn nested_styles_reach_every_leaf() {
        assert_eq!(
            render_colored("{red|plain **bold**}"),
            format!("{}{}", "plain ".red(), "bold".red().bold())
        );
    }

    #[test]
    fn nesting_order_does_not_matter() {
        assert_eq!(
            render_colored("**__{red|x}__**"),
            render_colored("{red|**__x__**}")
        );
    }

    #[test]
    fn dice_inherit_surrounding_styles() {
        /* A d1 always rolls 1, so 3d1 is deterministically 3. */
        assert_eq!(
            render_colored("{red|**[3d1]**}"),
            "3".red().bold().to_string()
        );
    }

    #[test]
    fn malformed_markup_renders_as_text() {
        assert_eq!(
            render_colored("**unclosed, {nope|x}, [see pg. 12]"),
            "**unclosed, {nope|x}, [see pg. 12]"
        );
    }

    #[test]
    fn hex_colour() {
        assert_eq!(
            render_colored("{#ff8800|x}"),
            "x".truecolor(255, 136, 0).to_string()
        );
    }
}
