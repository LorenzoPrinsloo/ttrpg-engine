/* A fingerprint of a config file's bytes, saved with a session so a later
   start can tell whether the file was edited in between.

   FNV-1a rather than std's `DefaultHasher`: that one's output may change
   between Rust releases, which would make every saved session look stale
   after a toolchain update. Spotting an edit needs no more than this. */
pub(crate) fn fingerprint(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let hash = bytes
        .iter()
        .fold(OFFSET, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(PRIME));

    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_published_fnv_1a_values() {
        /* From the FNV reference test vectors, so a saved hash means the same
           thing whichever build wrote it. */
        assert_eq!(fingerprint(b""), "cbf29ce484222325");
        assert_eq!(fingerprint(b"a"), "af63dc4c8601ec8c");
        assert_eq!(fingerprint(b"foobar"), "85944171f73967e8");
    }

    #[test]
    fn a_one_character_edit_changes_it() {
        assert_ne!(fingerprint(b"ratio = 0.19"), fingerprint(b"ratio = 0.18"));
    }
}
