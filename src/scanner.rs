use crate::{Action, Error};

/// A tool for processing the characters in a string individually and
/// in groups with only one character of lookahead.
#[derive(Debug)]
pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

impl Scanner {
    pub fn new(string: &str) -> Self {
        Self {
            cursor: 0,
            characters: string.chars().collect(),
        }
    }

    /// Returns the current cursor. Useful for reporting errors.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the next character without advancing the cursor.
    /// AKA "lookahead"
    pub fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    /// Returns true if further progress is not possible.
    pub fn is_done(&self) -> bool {
        self.cursor == self.characters.len()
    }

    /// Returns the next character (if available) and advances the cursor.
    pub fn pop(&mut self) -> Option<&char> {
        self.characters.get(self.cursor).map(|c| {
            self.cursor += 1;
            c
        })
    }

    /// Returns true if the `target` is found at the current cursor position,
    /// and advances the cursor.
    /// Otherwise, returns false leaving the cursor unchanged.
    pub fn take(&mut self, target: &char) -> bool {
        match self.characters.get(self.cursor) {
            Some(character) if character == target => {
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }

    /// Iteratively directs the advancement of the cursor and the return
    /// of translated values.
    pub fn scan<T>(
        &mut self,
        cb: impl Fn(&str) -> Option<Action<T>>,
    ) -> Result<Option<T>, Error> {
        let mut sequence = String::new();
        let mut require = false;
        let mut request = None;

        loop {
            match self.characters.get(self.cursor) {
                Some(target) => {
                    sequence.push(*target);

                    match cb(&sequence) {
                        Some(Action::Return(result)) => {
                            self.cursor += 1;

                            break Ok(Some(result));
                        }
                        Some(Action::Request(result)) => {
                            self.cursor += 1;
                            require = false;
                            request = Some(result);
                        }
                        Some(Action::Require) => {
                            self.cursor += 1;
                            require = true;
                        }
                        None => {
                            if require {
                                break Err(Error::Character(self.cursor));
                            } else {
                                break Ok(request);
                            }
                        }
                    }
                }
                None => {
                    if require {
                        break Err(Error::EndOfLine);
                    } else {
                        break Ok(request);
                    }
                }
            }
        }
    }

    /// Invoke `cb` once. If the result is not `None`, return it and advance
    /// the cursor. Otherwise, return None and leave the cursor unchanged.
    pub fn transform<T>(
        &mut self,
        cb: impl FnOnce(&char) -> Option<T>,
    ) -> Option<T> {
        self.characters.get(self.cursor).and_then(|c| {
            cb(c).map(|c| {
                self.cursor += 1;
                c
            })
        })
    }
}

#[cfg(test)]
mod cursor {
    use super::*;

    #[test]
    fn empty() {
        let scanner = Scanner::new("");

        assert_eq!(scanner.cursor(), 0)
    }

    #[test]
    fn in_progress() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();

        assert_eq!(scanner.cursor(), 1);
    }

    #[test]
    fn done() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();
        scanner.pop();
        scanner.pop();

        assert_eq!(scanner.cursor(), 3)
    }
}

#[cfg(test)]
mod is_done {
    use super::*;

    #[test]
    fn emtpy() {
        let scanner = Scanner::new("");

        assert!(scanner.is_done())
    }

    #[test]
    fn not_done() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();

        assert_eq!(scanner.is_done(), false)
    }

    #[test]
    fn done() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();
        scanner.pop();
        scanner.pop();

        assert!(scanner.is_done())
    }
}

#[cfg(test)]
mod peek {
    use super::*;

    #[test]
    fn empty() {
        let scanner = Scanner::new("");

        assert_eq!(scanner.peek(), None)
    }

    #[test]
    fn not_done() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();

        assert_eq!(scanner.peek(), Some(&'b'))
    }
}

#[cfg(test)]
mod pop {
    use super::*;

    #[test]
    fn empty() {
        let mut scanner = Scanner::new("");

        assert_eq!(scanner.pop(), None);
        assert_eq!(scanner.cursor(), 0)
    }

    #[test]
    fn not_done() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(scanner.pop(), Some(&'a'));
        assert_eq!(scanner.cursor(), 1)
    }

    #[test]
    fn done() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();
        scanner.pop();
        scanner.pop();

        assert_eq!(scanner.pop(), None);
        assert_eq!(scanner.cursor(), 3)
    }
}

#[cfg(test)]
mod take {
    use super::*;

    #[test]
    fn empty() {
        let mut scanner = Scanner::new("");

        assert_eq!(scanner.take(&'a'), false);
        assert_eq!(scanner.cursor(), 0)
    }

    #[test]
    fn unmatched() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(scanner.take(&'b'), false);
        assert_eq!(scanner.cursor(), 0)
    }

    #[test]
    fn matched() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();

        assert_eq!(scanner.take(&'b'), true);
        assert_eq!(scanner.cursor(), 2)
    }
}

#[cfg(test)]
mod scan {
    use super::*;

    #[test]
    fn empty() {
        let mut scanner = Scanner::new("");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                _ => None,
            } as Option<Action<()>>),
            Ok(None)
        );
        assert_eq!(scanner.cursor(), 0)
    }

    #[test]
    fn return_only() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Return(())),
                _ => unreachable!(),
            }),
            Ok(Some(()))
        );
        assert_eq!(scanner.cursor(), 1)
    }

    #[test]
    fn require_mismatch() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Require),
                _ => None,
            } as Option<Action<()>>),
            Err(Error::Character(1))
        );
        assert_eq!(scanner.cursor(), 1)
    }

    #[test]
    fn require_end_of_line() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "b" => Some(Action::Require),
                "bc" => Some(Action::Require),
                _ => None,
            } as Option<Action<()>>),
            Err(Error::EndOfLine)
        );
        assert_eq!(scanner.cursor(), 3)
    }

    #[test]
    fn require_match() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Require),
                "ab" => Some(Action::Require),
                "abc" => Some(Action::Return(())),
                _ => None,
            }),
            Ok(Some(()))
        );
        assert_eq!(scanner.cursor(), 3)
    }

    #[test]
    fn require_request_mismatch() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Require),
                "ab" => Some(Action::Request(())),
                _ => None,
            }),
            Ok(Some(()))
        );
        assert_eq!(scanner.cursor(), 2)
    }

    #[test]
    fn request_mismatch() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Request(())),
                _ => None,
            }),
            Ok(Some(()))
        );
        assert_eq!(scanner.cursor(), 1)
    }

    #[test]
    fn request_end_of_line() {
        let mut scanner = Scanner::new("abc");

        scanner.pop();
        scanner.pop();

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "c" => Some(Action::Request(())),
                _ => None,
            }),
            Ok(Some(()))
        );
        assert_eq!(scanner.cursor(), 3)
    }

    #[test]
    fn request_match() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Request(1)),
                "ab" => Some(Action::Return(2)),
                _ => unreachable!(),
            }),
            Ok(Some(2))
        );
        assert_eq!(scanner.cursor(), 2)
    }

    #[test]
    fn request_require_mismatch() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Request(1)),
                "ab" => Some(Action::Require),
                _ => None,
            }),
            Err(Error::Character(2))
        );
        assert_eq!(scanner.cursor(), 2)
    }

    #[test]
    fn request_require_match() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(
            scanner.scan(|sequence| match sequence {
                "a" => Some(Action::Request(1)),
                "ab" => Some(Action::Require),
                "abc" => Some(Action::Return(2)),
                _ => None,
            }),
            Ok(Some(2))
        );
        assert_eq!(scanner.cursor(), 3)
    }
}

#[cfg(test)]
mod transform {
    use super::*;

    #[test]
    fn empty() {
        let mut scanner = Scanner::new("");

        assert_eq!(scanner.transform(|_| Some(1)), None);
        assert_eq!(scanner.cursor(), 0)
    }

    #[test]
    fn unmatched() {
        let mut scanner = Scanner::new("abc");
        let result: Option<usize> = None;

        assert_eq!(scanner.transform(|_| result), None);
    }

    #[test]
    fn matched() {
        let mut scanner = Scanner::new("abc");

        assert_eq!(scanner.transform(|_| Some(1)), Some(1))
    }
}
