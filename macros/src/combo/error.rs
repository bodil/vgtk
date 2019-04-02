use std::cmp::{Ord, Ordering};
use std::collections::BTreeSet;
use std::fmt::{Display, Error as FmtError, Formatter};

use super::Cursor;

#[derive(Debug)]
pub struct Error<'a, Input> {
    pub input: Cursor<'a, Input>,
    pub expected: BTreeSet<String>,
    pub fatal: bool,
    pub description: Option<String>,
}

impl<'a, I> Display for Error<'a, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let expected: Vec<String> = self.expected.iter().cloned().collect();
        write!(f, "expected {}", expected.join(", "))
    }
}

impl<'a, Input> Error<'a, Input> {
    pub fn new(input: &Cursor<'a, Input>) -> Self {
        Error {
            input: input.clone(),
            expected: BTreeSet::new(),
            fatal: false,
            description: None,
        }
    }

    pub fn fatal(mut self) -> Self {
        self.fatal = true;
        self
    }

    pub fn describe<S: Into<String>>(mut self, s: S) -> Self {
        self.description = Some(s.into());
        self
    }

    pub fn expect<S: Into<String>>(mut self, s: S) -> Self {
        self.expected.insert(s.into());
        self
    }

    pub fn expect_from<I, S>(mut self, expected: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.expected.extend(expected.into_iter().map(Into::into));
        self
    }

    pub fn is_fatal(&self) -> bool {
        self.fatal
    }

    pub fn extend(self, other: Self) -> Self {
        match self.input.cmp(&other.input) {
            Ordering::Greater => self,
            Ordering::Less => other,
            Ordering::Equal => self.expect_from(other.expected),
        }
    }
}
