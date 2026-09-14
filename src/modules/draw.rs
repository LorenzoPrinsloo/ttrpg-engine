/// A collection of entries that yields one at random.
///
/// Implemented by `Bag` and `Table`, which differ in whether a draw takes
/// what it returns out of play.
pub trait Draw {
    type Item;

    fn draw(&mut self) -> Option<Self::Item>;

    /// Puts every entry back in play.
    fn fill(&mut self);

    fn remaining(&self) -> usize;

    /// Whether an entry can come up at most once before a refill.
    fn unique(&self) -> bool;

    /// Whether drawing depletes it, so `remaining` counts down instead of
    /// reporting a fixed number of rows. A bag always does, because a draw
    /// takes a marble out; a table only when it is unique.
    fn exhausts(&self) -> bool;
}
