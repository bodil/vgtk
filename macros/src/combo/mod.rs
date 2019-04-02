mod boxparser;
mod combinators;
mod cursor;
mod error;
mod parser;
mod stream;
mod success;

pub use self::boxparser::BoxParser;
pub use self::combinators::*;
pub use self::cursor::Cursor;
pub use self::error::Error;
pub use self::parser::Parser;
pub use self::stream::Stream;
pub use self::success::Success;

pub type ParseResult<'a, Input, Output> = Result<Success<'a, Input, Output>, Error<'a, Input>>;

pub fn ok<'a, Input, Output>(
    value: Output,
    start: &Cursor<'a, Input>,
    next: Cursor<'a, Input>,
) -> ParseResult<'a, Input, Output> {
    Ok(Success::new(value, start, next))
}

pub fn err<'a, Input, Output, S>(
    input: &Cursor<'a, Input>,
    expect: S,
) -> ParseResult<'a, Input, Output>
where
    S: Into<String>,
{
    Err(Error::new(input).expect(expect))
}
