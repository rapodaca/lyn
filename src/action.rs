#[derive(Debug, PartialEq, Clone)]
pub enum Action<T> {
    /// If next iteration returns None, return T without advancing
    /// the cursor.
    Request(T),

    /// If the next iteration returns None, return None without advancing
    /// the cursor.
    Require,

    /// Immediately advance the cursor and return T.
    Return(T),
}
