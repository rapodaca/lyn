#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// An unexpected end-of-line has been found.
    EndOfLine,

    /// A syntax error at the indicated cursor position has been found.
    Character(usize),
}
