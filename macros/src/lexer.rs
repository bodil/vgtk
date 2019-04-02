use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

use std::fmt::{Display, Error, Formatter};
use std::iter::FromIterator;

#[derive(Clone, Debug)]
pub enum Token {
    Ident(Ident),
    Literal(Literal),
    Punct(char, Punct),
    Group(Delimiter, Group),
    GroupOpen(Delimiter, Span),
    GroupClose(Delimiter, Span),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Ident(ident) => ident.span(),
            Token::Literal(literal) => literal.span(),
            Token::Punct(_, punct) => punct.span(),
            Token::Group(_, group) => group.span(),
            Token::GroupOpen(_, span) => *span,
            Token::GroupClose(_, span) => *span,
        }
    }

    pub fn is_ident(&self) -> bool {
        match self {
            Token::Ident(_) => true,
            _ => false,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Token::Ident(ident) => ident.fmt(f),
            Token::Literal(literal) => literal.fmt(f),
            Token::Punct(_, punct) => punct.fmt(f),
            Token::Group(_, group) => group.fmt(f),
            Token::GroupOpen(del, _) => write!(f, "{}", render_delim(*del, false)),
            Token::GroupClose(del, _) => write!(f, "{}", render_delim(*del, true)),
        }
    }
}

impl<'a> From<&'a Token> for TokenTree {
    fn from(token: &'a Token) -> Self {
        match token {
            Token::Ident(ident) => TokenTree::Ident(ident.clone()),
            Token::Literal(literal) => TokenTree::Literal(literal.clone()),
            Token::Punct(_, punct) => TokenTree::Punct(punct.clone()),
            Token::Group(_, group) => TokenTree::Group(group.clone()),
            Token::GroupOpen(_, _) => panic!("Can't convert a GroupOpen token to a TokenTree"),
            Token::GroupClose(_, _) => panic!("Can't convert a GroupClose token to a TokenTree"),
        }
    }
}

impl From<Token> for TokenTree {
    fn from(token: Token) -> Self {
        TokenTree::from(&token)
    }
}

impl From<Token> for TokenStream {
    fn from(token: Token) -> Self {
        TokenStream::from_iter(vec![TokenTree::from(token)])
    }
}

impl From<Ident> for Token {
    fn from(ident: Ident) -> Self {
        Token::Ident(ident)
    }
}

impl From<Literal> for Token {
    fn from(literal: Literal) -> Self {
        Token::Literal(literal)
    }
}

impl From<Punct> for Token {
    fn from(punct: Punct) -> Self {
        Token::Punct(punct.as_char(), punct)
    }
}

impl From<Group> for Token {
    fn from(group: Group) -> Self {
        Token::Group(group.delimiter(), group)
    }
}

pub fn render_delim(delim: Delimiter, closing: bool) -> &'static str {
    if closing {
        match delim {
            Delimiter::Parenthesis => ")",
            Delimiter::Brace => "}",
            Delimiter::Bracket => "]",
            Delimiter::None => unimplemented!(),
        }
    } else {
        match delim {
            Delimiter::Parenthesis => "(",
            Delimiter::Brace => "{",
            Delimiter::Bracket => "[",
            Delimiter::None => unimplemented!(),
        }
    }
}

pub fn to_stream<'a, I: IntoIterator<Item = &'a Token>>(tokens: I) -> TokenStream {
    let mut stream = TokenStream::new();
    stream.extend(tokens.into_iter().map(TokenTree::from));
    stream
}

pub fn unroll_stream(stream: TokenStream, deep: bool) -> Vec<Token> {
    let mut vec = Vec::new();
    for tt in stream {
        match tt {
            TokenTree::Ident(ident) => vec.push(ident.into()),
            TokenTree::Literal(literal) => vec.push(literal.into()),
            TokenTree::Punct(punct) => vec.push(punct.into()),
            TokenTree::Group(ref group) if deep && group.delimiter() != Delimiter::Parenthesis => {
                vec.push(Token::GroupOpen(group.delimiter(), group.span()));
                let sub = unroll_stream(group.stream(), deep);
                vec.extend(sub);
                vec.push(Token::GroupClose(group.delimiter(), group.span()));
            }
            TokenTree::Group(group) => vec.push(group.into()),
        }
    }
    vec
}
