use crate::encounters::Die;

// Sanity caps, so `[9999999d9999999]` cannot be used as a resource bomb.
// They also keep the `u32` sum inside [`Die::roll`] far from overflowing.
const MAX_DICE: u32 = 1_000;
const MAX_SIDES: u32 = 1_000_000;

// The contents of a `[...]` span: a signed sum of terms, as in `2d6+3-1d4`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct DiceExpr {
    pub terms: Vec<(Sign, Term)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sign {
    Add,
    Sub,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    Roll(Die),
    Const(u32),
}

impl DiceExpr {
    pub fn parse(raw: &str) -> Option<Self> {
        let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();

        let mut terms = Vec::new();
        let mut sign = Sign::Add;
        let mut start = 0;
        for (i, c) in compact.char_indices() {
            let next = match c {
                '+' => Sign::Add,
                '-' => Sign::Sub,
                _ => continue,
            };
            terms.push((sign, Term::parse(&compact[start..i])?));
            sign = next;
            start = i + 1;
        }
        terms.push((sign, Term::parse(&compact[start..])?));

        Some(Self { terms })
    }

    pub fn eval(&self) -> i64 {
        self.terms
            .iter()
            .map(|(sign, term)| {
                let value = i64::from(term.eval());
                match sign {
                    Sign::Add => value,
                    Sign::Sub => -value,
                }
            })
            .sum()
    }
}

impl Term {
    fn parse(text: &str) -> Option<Self> {
        let Some((count, sides)) = text.split_once(['d', 'D']) else {
            // `parse` rejects empty strings and non-ASCII digits for us.
            return Some(Self::Const(text.parse().ok()?));
        };

        let die = Die {
            n: count.parse().ok()?,
            sides: sides.parse().ok()?,
        };
        let within_caps = (1..=MAX_DICE).contains(&die.n) && (1..=MAX_SIDES).contains(&die.sides);

        within_caps.then_some(Self::Roll(die))
    }

    fn eval(self) -> u32 {
        match self {
            Self::Roll(die) => die.roll(),
            Self::Const(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Sign::{Add, Sub};

    fn roll(n: u32, sides: u32) -> (Sign, Term) {
        (Add, Term::Roll(Die { n, sides }))
    }

    #[test]
    fn single_roll() {
        assert_eq!(
            DiceExpr::parse("2d6"),
            Some(DiceExpr {
                terms: vec![roll(2, 6)]
            })
        );
    }

    #[test]
    fn signed_sum_of_terms() {
        assert_eq!(
            DiceExpr::parse("1d8 + 2d4 - 1"),
            Some(DiceExpr {
                terms: vec![roll(1, 8), roll(2, 4), (Sub, Term::Const(1))],
            })
        );
    }

    #[test]
    fn uppercase_d_is_accepted() {
        assert_eq!(DiceExpr::parse("1D20"), DiceExpr::parse("1d20"));
    }

    #[test]
    fn malformed_expressions_are_rejected() {
        for raw in [
            "",
            "d6",
            "2d",
            "2d6d6",
            "see pg. 12",
            "1d6+",
            "+1d6",
            "1d6++1",
            "1,5",
            "2 d 6+x",
        ] {
            assert_eq!(DiceExpr::parse(raw), None, "{raw:?} should not parse");
        }
    }

    #[test]
    fn dice_outside_the_caps_are_rejected() {
        assert_eq!(DiceExpr::parse("0d6"), None);
        assert_eq!(DiceExpr::parse("1d0"), None);
        assert_eq!(DiceExpr::parse("1001d6"), None);
        assert_eq!(DiceExpr::parse("1d1000001"), None);
        assert!(DiceExpr::parse("1000d1000000").is_some());
    }

    #[test]
    fn oversized_numbers_are_rejected_not_wrapped() {
        assert_eq!(DiceExpr::parse("99999999999d6"), None);
        assert_eq!(DiceExpr::parse("99999999999"), None);
    }

    #[test]
    fn eval_sums_rolls_and_constants() {
        // A d1 always rolls 1, so these totals are deterministic.
        assert_eq!(DiceExpr::parse("3d1+2").unwrap().eval(), 5);
        assert_eq!(DiceExpr::parse("2-5d1").unwrap().eval(), -3);
    }

    #[test]
    fn eval_stays_within_range() {
        let expr = DiceExpr::parse("2d6+3").unwrap();
        for _ in 0..100 {
            assert!((5..=15).contains(&expr.eval()));
        }
    }
}
