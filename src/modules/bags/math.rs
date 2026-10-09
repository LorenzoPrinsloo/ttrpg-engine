pub(super) fn greatest_common_denominator(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        greatest_common_denominator(b, a % b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_leaves_the_other_side_untouched() {
        // Folding a set of weights starts from 0, so this is the identity
        // case that makes the fold work.
        assert_eq!(greatest_common_denominator(0, 7), 7);
        assert_eq!(greatest_common_denominator(7, 0), 7);
    }

    #[test]
    fn common_and_coprime_factors() {
        assert_eq!(greatest_common_denominator(12, 18), 6);
        assert_eq!(greatest_common_denominator(50, 100), 50);
        assert_eq!(greatest_common_denominator(19, 100), 1);
    }

    #[test]
    fn values_beyond_a_byte() {
        assert_eq!(greatest_common_denominator(1_000, 250), 250);
        assert_eq!(greatest_common_denominator(900, 1_000), 100);
    }
}
