use std::fmt;
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

// A row's range as written in the file: `"01-19"`, or `"20"` for a single
// number. The spelling is kept so errors can quote what was typed rather
// than Rust's `1..=19`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RowRange {
    raw: String,
    span: RangeInclusive<u32>,
}

impl From<RowRange> for String {
    fn from(row_range: RowRange) -> Self {
        row_range.raw
    }
}

impl RowRange {
    pub fn span(&self) -> RangeInclusive<u32> {
        self.span.clone()
    }
}

impl TryFrom<String> for RowRange {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let trimmed = raw.trim();
        let (start, end) = match trimmed.split_once('-') {
            Some((start, end)) => (start.trim(), end.trim()),
            None => (trimmed, trimmed),
        };

        let number = |text: &str| {
            text.parse::<u32>()
                .map_err(|_| format!("{raw:?} is not a range, write it as \"01-19\" or \"20\""))
        };
        let span = number(start)?..=number(end)?;

        if span.start() > span.end() {
            return Err(format!("the range {raw:?} runs backwards"));
        }

        Ok(Self { raw, span })
    }
}

impl From<RangeInclusive<u32>> for RowRange {
    fn from(span: RangeInclusive<u32>) -> Self {
        let raw = match span.start() == span.end() {
            true => span.start().to_string(),
            false => format!("{}-{}", span.start(), span.end()),
        };

        Self { raw, span }
    }
}

impl fmt::Display for RowRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Result<RowRange, String> {
        RowRange::try_from(raw.to_string())
    }

    #[test]
    fn a_span_of_numbers() {
        assert_eq!(parse("1-19").unwrap().span(), 1..=19);
    }

    #[test]
    fn leading_zeros_and_spaces_are_tolerated() {
        // d100 rows get copied straight out of a book.
        assert_eq!(parse("01-19").unwrap().span(), 1..=19);
        assert_eq!(parse(" 7 ").unwrap().span(), 7..=7);
    }

    #[test]
    fn a_single_number_is_one_row() {
        assert_eq!(parse("20").unwrap().span(), 20..=20);
    }

    #[test]
    fn the_raw_spelling_is_kept_for_error_messages() {
        assert_eq!(parse("01-19").unwrap().to_string(), "01-19");
    }

    #[test]
    fn malformed_ranges_are_rejected() {
        for raw in ["", "-", "1--9", "a-b", "1-", "-9", "4294967296"] {
            assert!(parse(raw).is_err(), "{raw:?} should not parse");
        }
    }

    #[test]
    fn a_backwards_range_says_so() {
        let error = parse("19-1").err().unwrap();

        assert!(error.contains("backwards"), "{error}");
    }
}
