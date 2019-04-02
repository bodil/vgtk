use std::ops::Deref;

use super::Cursor;

pub trait Stream<'a, Input: 'a> {
    fn get(&'a self, index: usize) -> Option<&'a Input>;
    fn cursor(&'a self) -> Cursor<'a, Input>
    where
        Self: Sized,
    {
        Cursor::new(self, 0)
    }
}

impl<'a, Input: 'a, Slice> Stream<'a, Input> for Slice
where
    Slice: Deref<Target = [Input]>,
{
    fn get(&'a self, index: usize) -> Option<&'a Input> {
        self.deref().get(index)
    }
}
