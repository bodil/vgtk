use std::ops::{Bound, RangeBounds};

use super::{err, ok, BoxParser, Cursor, Error, Parser, Success};

pub fn succeed<'a, F, Input, Output>(lazy_result: F) -> impl Parser<'a, Input, Output>
where
    Input: 'a,
    Output: 'a,
    F: Fn() -> Output + 'a,
{
    move |input: &Cursor<'a, Input>| ok(lazy_result(), input, input.clone())
}

pub fn fail<'a, Input, Output, F>(lazy_error: F) -> impl Parser<'a, Input, Output>
where
    Input: 'a,
    Output: 'a,
    F: Fn() -> Error<'a, Input> + 'a,
{
    move |_input: &Cursor<'a, Input>| Err(lazy_error())
}

pub fn end<'a, Input>() -> impl Parser<'a, Input, ()>
where
    Input: 'a,
{
    |input: &Cursor<'a, Input>| match input.get() {
        None => ok((), input, input.clone()),
        Some(_) => err(input, "end of input"),
    }
}

pub fn one<'a, Input>() -> impl Parser<'a, Input, Input>
where
    Input: Clone + 'a,
{
    |input: &Cursor<'a, Input>| {
        input
            .get()
            .map(|v| Success::new(v.clone(), input, input.next()))
            .ok_or_else(|| Error::new(input).expect("anything"))
    }
}

pub fn optional_with<'a, P, Input, Output, FP, FE, OptionalOutput>(
    parser: P,
    present: FP,
    empty: FE,
) -> impl Parser<'a, Input, OptionalOutput>
where
    Input: 'a,
    Output: 'a,
    OptionalOutput: 'a,
    P: Parser<'a, Input, Output> + 'a,
    FP: Fn(Output) -> OptionalOutput + 'a,
    FE: Fn() -> OptionalOutput + 'a,
{
    move |input: &Cursor<'a, Input>| match parser.parse(input) {
        Ok(success) => Ok(success.map(&present)),
        Err(_) => ok(empty(), input, input.clone()),
    }
}

pub fn optional<'a, P, Input, Output>(parser: P) -> impl Parser<'a, Input, Option<Output>>
where
    Input: 'a,
    Output: 'a,
    P: Parser<'a, Input, Output> + 'a,
{
    optional_with(parser, |v| Some(v), || None)
}

pub fn map<'a, P, Input, I, O, F>(parser: P, f: F) -> impl Parser<'a, Input, O>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, I> + 'a,
    F: Fn(I) -> O + 'a,
{
    move |input: &Cursor<'a, Input>| match parser.parse(input) {
        Ok(success) => Ok(success.map(&f)),
        Err(error) => Err(error),
    }
}

pub fn seq<'a, P, F, Input, I, O, PO>(parser: P, f: F) -> impl Parser<'a, Input, O>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, I> + 'a,
    F: Fn(I) -> PO + 'a,
    PO: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| match parser.parse(input) {
        Ok(success) => f(success.value)
            .parse(&success.next)
            .map(|success| success.with_start(input)),
        Err(error) => Err(error),
    }
}

pub fn pair<'a, P1, P2, I, O1, O2>(p1: P1, p2: P2) -> impl Parser<'a, I, (O1, O2)>
where
    I: 'a,
    O1: 'a,
    O2: 'a,
    P1: Parser<'a, I, O1> + 'a,
    P2: Parser<'a, I, O2> + 'a,
{
    move |input: &Cursor<'a, I>| match p1.parse(input) {
        Ok(success1) => match p2.parse(&success1.next) {
            Ok(success2) => Ok(success2.map(|o2| (success1.value, o2)).with_start(input)),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    }
}

pub fn left<'a, P1, P2, I, O1, O2>(p1: P1, p2: P2) -> impl Parser<'a, I, O1>
where
    I: 'a,
    O1: 'a,
    O2: 'a,
    P1: Parser<'a, I, O1> + 'a,
    P2: Parser<'a, I, O2> + 'a,
{
    move |input: &Cursor<'a, I>| match p1.parse(input) {
        Ok(success1) => match p2.parse(&success1.next) {
            Ok(success2) => Ok(success2.map(|_| success1.value).with_start(input)),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    }
}

pub fn right<'a, P1, P2, I, O1, O2>(p1: P1, p2: P2) -> impl Parser<'a, I, O2>
where
    I: 'a,
    O1: 'a,
    O2: 'a,
    P1: Parser<'a, I, O1> + 'a,
    P2: Parser<'a, I, O2> + 'a,
{
    move |input: &Cursor<'a, I>| match p1.parse(input) {
        Ok(success1) => match p2.parse(&success1.next) {
            Ok(success) => Ok(success.with_start(input)),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    }
}

pub fn expect<'a, P, Input, O>(parser: P) -> impl Parser<'a, Input, O>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| match parser.parse(input) {
        result @ Ok(_) => result,
        Err(error) => Err(error.fatal()),
    }
}

pub fn assert<'a, P, I, O, F, ErrF>(parser: P, assertion: F) -> impl Parser<'a, I, O>
where
    I: 'a,
    O: 'a,
    P: Parser<'a, I, O> + 'a,
    F: Fn(O) -> Result<O, ErrF> + 'a,
    ErrF: FnOnce(Error<'a, I>) -> Error<'a, I>,
{
    move |input: &Cursor<'a, I>| match parser.parse(input) {
        Ok(success) => match assertion(success.value) {
            Ok(result) => ok(result, &success.start, success.next),
            Err(err_fn) => Err(err_fn(Error::new(input))),
        },
        err => err,
    }
}

pub fn cut_seq<'a, P, F, Input, I, O, PO>(parser: P, f: F) -> impl Parser<'a, Input, O>
where
    Input: 'a,
    P: Parser<'a, Input, I> + 'a,
    F: Fn(I) -> PO + 'a,
    PO: Parser<'a, Input, O> + 'a,
    O: 'a,
{
    seq(parser, move |result| expect(f(result)))
}

pub fn either<'a, P1, P2, Input, O>(p1: P1, p2: P2) -> impl Parser<'a, Input, O>
where
    Input: 'a,
    O: 'a,
    P1: Parser<'a, Input, O> + 'a,
    P2: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| {
        p1.parse(input).or_else(|err| {
            if err.is_fatal() {
                Err(err)
            } else {
                p2.parse(input).map_err(|err2| err.extend(err2))
            }
        })
    }
}

pub fn exactly<'a, P, Input, O>(parser: P, count: usize) -> impl Parser<'a, Input, Vec<O>>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| {
        let mut out = Vec::new();
        let mut i = count;
        let mut cursor = input.clone();
        while i > 0 {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(err) => return Err(err),
            }
            i -= 1;
        }
        ok(out, input, cursor)
    }
}

pub fn up_to<'a, P, Input, O>(parser: P, count: usize) -> impl Parser<'a, Input, Vec<O>>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| {
        let mut out = Vec::new();
        let mut i = count;
        let mut cursor = input.clone();
        while i > 0 {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(_) => break,
            }
            i -= 1;
        }
        ok(out, input, cursor)
    }
}

pub fn any<'a, P, Input, O>(parser: P) -> impl Parser<'a, Input, Vec<O>>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| {
        let mut out = Vec::new();
        let mut cursor = input.clone();
        loop {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(_) => break,
            }
        }
        ok(out, input, cursor)
    }
}

pub fn at_least<'a, P, Input, O>(parser: P, count: usize) -> impl Parser<'a, Input, Vec<O>>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| {
        let mut out = Vec::new();
        let mut i = count;
        let mut cursor = input.clone();
        while i > 0 {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(err) => return Err(err),
            }
            i -= 1;
        }
        loop {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(_) => break,
            }
        }
        ok(out, input, cursor)
    }
}

pub fn between<'a, P, Input, O>(parser: P, min: usize, max: usize) -> impl Parser<'a, Input, Vec<O>>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
{
    move |input: &Cursor<'a, Input>| {
        let mut out = Vec::new();
        let mut i = min;
        let mut cursor = input.clone();
        while i > 0 {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(err) => return Err(err),
            }
            i -= 1;
        }
        i = max - min;
        while i > 0 {
            match parser.parse(&cursor) {
                Ok(success) => {
                    out.push(success.value);
                    cursor = success.next;
                }
                Err(_) => break,
            }
            i -= 1;
        }
        ok(out, input, cursor)
    }
}

pub fn repeat<'a, P, Input, O, R>(parser: P, range: R) -> BoxParser<'a, Input, Vec<O>>
where
    Input: 'a,
    O: 'a,
    P: Parser<'a, Input, O> + 'a,
    R: RangeBounds<usize> + 'a,
{
    let min = match range.start_bound() {
        Bound::Included(min) => *min,
        Bound::Excluded(_) => unreachable!(),
        Bound::Unbounded => 0,
    };
    let max = match range.end_bound() {
        Bound::Included(max) => Some(*max + 1),
        Bound::Excluded(max) => Some(*max),
        Bound::Unbounded => None,
    }
    .map(|max| max - min);
    match (min, max) {
        (0, None) => any(parser).to_box(),
        (0, Some(max)) => up_to(parser, max).to_box(),
        (min, Some(max)) if min == max => exactly(parser, min).to_box(),
        (min, None) => at_least(parser, min).to_box(),
        (min, Some(max)) => between(parser, min, max).to_box(),
    }
}

pub fn sep_by<'a, P, SP, I, O>(parser: P, sep: SP) -> impl Parser<'a, I, Vec<O>>
where
    I: 'a,
    O: 'a,
    P: Parser<'a, I, O> + 'a,
    SP: Parser<'a, I, O> + 'a,
{
    move |input: &Cursor<'a, I>| match parser.parse(input) {
        Err(err) => Err(err),
        Ok(head) => {
            let mut cursor = head.next;
            let mut out = vec![head.value];
            loop {
                match sep.parse(&cursor) {
                    Ok(success) => {
                        out.push(success.value);
                        cursor = success.next;
                    }
                    Err(_) => break,
                }
                match parser.parse(&cursor) {
                    Ok(success) => {
                        out.push(success.value);
                        cursor = success.next;
                    }
                    Err(_) => break,
                }
            }
            ok(out, input, cursor)
        }
    }
}
