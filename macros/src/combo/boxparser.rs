use std::ops::{BitOr, Shr};

use super::{Cursor, ParseResult, Parser};

pub struct BoxParser<'a, Input, Output> {
    parser: Box<dyn Parser<'a, Input, Output> + 'a>,
}

impl<'a, Input: 'a, Output: 'a> BoxParser<'a, Input, Output> {
    pub fn new<P>(p: P) -> Self
    where
        P: Parser<'a, Input, Output> + 'a,
    {
        BoxParser {
            parser: Box::new(p),
        }
    }
}

impl<'a, Input, Output> Parser<'a, Input, Output> for BoxParser<'a, Input, Output> {
    #[inline]
    fn parse(&self, input: &Cursor<'a, Input>) -> ParseResult<'a, Input, Output> {
        self.parser.parse(input)
    }

    fn to_box(self) -> Self {
        self
    }
}

impl<'a, Input, Output, P> BitOr<P> for BoxParser<'a, Input, Output>
where
    Input: 'a,
    Output: 'a,
    P: Parser<'a, Input, Output> + 'a,
{
    type Output = BoxParser<'a, Input, Output>;

    fn bitor(self, next_parser: P) -> Self::Output {
        self.or(next_parser)
    }
}

impl<'a, Input, In, Out, F> Shr<F> for BoxParser<'a, Input, In>
where
    Input: 'a,
    In: 'a,
    Out: 'a,
    F: Fn(In) -> BoxParser<'a, Input, Out> + 'a,
{
    type Output = BoxParser<'a, Input, Out>;

    fn shr(self, f: F) -> Self::Output {
        self.and_then(f)
    }
}
