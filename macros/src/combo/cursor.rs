use std::cmp::Ordering;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::ops::{Range, Sub};

use super::Stream;

pub struct Cursor<'a, Input> {
    buffer: &'a dyn Stream<'a, Input>,
    cursor: usize,
}

impl<'a, Input> Cursor<'a, Input> {
    pub fn new(buffer: &'a dyn Stream<'a, Input>, cursor: usize) -> Self {
        Cursor { buffer, cursor }
    }

    pub fn next(&self) -> Self {
        Cursor {
            buffer: self.buffer,
            cursor: self.cursor + 1,
        }
    }

    pub fn get(&self) -> Option<&'a Input> {
        self.buffer.get(self.cursor)
    }
}

impl<'a, Input> Sub for Cursor<'a, Input> {
    type Output = Range<usize>;

    fn sub(self, other: Self) -> Self::Output {
        if self.cursor > other.cursor {
            Range {
                start: self.cursor,
                end: other.cursor,
            }
        } else {
            Range {
                start: other.cursor,
                end: self.cursor,
            }
        }
    }
}

impl<'a, Input> Clone for Cursor<'a, Input> {
    fn clone(&self) -> Self {
        Cursor {
            buffer: self.buffer,
            cursor: self.cursor,
        }
    }
}

impl<'a, Input> Debug for Cursor<'a, Input>
where
    Input: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}: ", self.cursor)?;
        match self.get() {
            Some(value) => write!(f, "{:?}", value),
            None => write!(f, "end of input"),
        }
    }
}

impl<'a, Input> PartialEq for Cursor<'a, Input> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.buffer, other.buffer) && self.cursor == other.cursor
    }
}

impl<'a, Input> Eq for Cursor<'a, Input> {}

impl<'a, Input> PartialOrd for Cursor<'a, Input> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        debug_assert!(
            std::ptr::eq(self.buffer, other.buffer),
            "cannot compare cursors for different buffers"
        );
        self.cursor.partial_cmp(&other.cursor)
    }
}

impl<'a, Input> Ord for Cursor<'a, Input> {
    fn cmp(&self, other: &Self) -> Ordering {
        debug_assert!(
            std::ptr::eq(self.buffer, other.buffer),
            "cannot compare cursors for different buffers"
        );
        self.cursor.cmp(&other.cursor)
    }
}
