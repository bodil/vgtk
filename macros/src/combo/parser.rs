use std::ops::RangeBounds;

use super::combinators::*;
use super::{BoxParser, Cursor, Error, ParseResult};

pub trait Parser<'a, Input, Output> {
    fn parse(&self, input: &Cursor<'a, Input>) -> ParseResult<'a, Input, Output>;
    fn to_box(self) -> BoxParser<'a, Input, Output>;

    fn map<F, O2>(self, f: F) -> BoxParser<'a, Input, O2>
    where
        Self: Sized + 'a,
        Input: 'a,
        F: Fn(Output) -> O2 + 'a,
        O2: 'a,
    {
        map(self, f).to_box()
    }

    fn and_then<F, O2, PO2>(self, f: F) -> BoxParser<'a, Input, O2>
    where
        Self: Sized + 'a,
        Input: 'a,
        O2: 'a,
        F: Fn(Output) -> PO2 + 'a,
        PO2: Parser<'a, Input, O2> + 'a,
    {
        seq(self, f).to_box()
    }

    fn or<P2>(self, next_parser: P2) -> BoxParser<'a, Input, Output>
    where
        Self: Sized + 'a,
        Input: 'a,
        Output: 'a,
        P2: Parser<'a, Input, Output> + 'a,
    {
        either(self, next_parser).to_box()
    }

    fn pair<P2, Output2>(self, next_parser: P2) -> BoxParser<'a, Input, (Output, Output2)>
    where
        Input: 'a,
        Output: 'a,
        Output2: 'a,
        Self: Sized + 'a,
        P2: Parser<'a, Input, Output2> + 'a,
    {
        pair(self, next_parser).to_box()
    }

    fn left<P2, Output2>(self, next_parser: P2) -> BoxParser<'a, Input, Output>
    where
        Input: 'a,
        Output: 'a,
        Output2: 'a,
        Self: Sized + 'a,
        P2: Parser<'a, Input, Output2> + 'a,
    {
        left(self, next_parser).to_box()
    }

    fn right<P2, Output2>(self, next_parser: P2) -> BoxParser<'a, Input, Output2>
    where
        Input: 'a,
        Output: 'a,
        Output2: 'a,
        Self: Sized + 'a,
        P2: Parser<'a, Input, Output2> + 'a,
    {
        right(self, next_parser).to_box()
    }

    fn cut<F, O2, PO2>(self, f: F) -> BoxParser<'a, Input, O2>
    where
        Self: Sized + 'a,
        Input: 'a,
        O2: 'a,
        F: Fn(Output) -> PO2 + 'a,
        PO2: Parser<'a, Input, O2> + 'a,
    {
        cut_seq(self, f).to_box()
    }

    fn expect(self) -> BoxParser<'a, Input, Output>
    where
        Input: 'a,
        Output: 'a,
        Self: Sized + 'a,
    {
        expect(self).to_box()
    }

    fn assert<F, ErrF>(self, assertion: F) -> BoxParser<'a, Input, Output>
    where
        Input: 'a,
        Output: 'a,
        Self: Sized + 'a,
        F: Fn(Output) -> Result<Output, ErrF> + 'a,
        ErrF: FnOnce(Error<'a, Input>) -> Error<'a, Input>,
    {
        assert(self, assertion).to_box()
    }

    fn repeat<R>(self, range: R) -> BoxParser<'a, Input, Vec<Output>>
    where
        Input: 'a,
        Output: 'a,
        Self: Sized + 'a,
        R: RangeBounds<usize> + 'a,
    {
        repeat(self, range).to_box()
    }

    fn sep_by<P>(self, separator: P) -> BoxParser<'a, Input, Vec<Output>>
    where
        Input: 'a,
        Output: 'a,
        Self: Sized + 'a,
        P: Parser<'a, Input, Output> + 'a,
    {
        sep_by(self, separator).to_box()
    }

    fn optional(self) -> BoxParser<'a, Input, Option<Output>>
    where
        Input: 'a,
        Output: 'a,
        Self: Sized + 'a,
    {
        optional(self).to_box()
    }

    fn optional_with<FP, FE, OptionalOutput>(
        self,
        present: FP,
        empty: FE,
    ) -> BoxParser<'a, Input, OptionalOutput>
    where
        Input: 'a,
        Output: 'a,
        OptionalOutput: 'a,
        Self: Sized + 'a,
        FP: Fn(Output) -> OptionalOutput + 'a,
        FE: Fn() -> OptionalOutput + 'a,
    {
        optional_with(self, present, empty).to_box()
    }
}

impl<'a, F, Input, Output> Parser<'a, Input, Output> for F
where
    Input: 'a,
    Output: 'a,
    F: Fn(&Cursor<'a, Input>) -> ParseResult<'a, Input, Output> + 'a,
{
    #[inline]
    fn parse(&self, input: &Cursor<'a, Input>) -> ParseResult<'a, Input, Output> {
        self(input)
    }

    fn to_box(self) -> BoxParser<'a, Input, Output> {
        BoxParser::new(self)
    }
}
