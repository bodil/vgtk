use std::fmt::{Display, Error, Formatter};
use std::iter::FromIterator;
use std::ops::{Add, Deref, DerefMut};

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::error::RsxParseError;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug, Clone)]
pub enum Keyword {
    Async,
    On,
}

fn keywordise(token: Token) -> Token {
    match token {
        Token::Ident(ident) => match ident.to_string().as_str() {
            "async" => Token::Keyword(Keyword::Async, ident),
            "on" => Token::Keyword(Keyword::On, ident),
            _ => Token::Ident(ident),
        },
        _ => token,
    }
}

#[derive(Clone, Debug)]
pub enum Token {
    Ident(Ident),
    Literal(Literal),
    Punct1(char, Punct),
    Punct2(char, char, Punct, Punct),
    Punct3(char, char, char, Punct, Punct, Punct),
    Group(Delimiter, Group),
    Keyword(Keyword, Ident),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Ident(ident) => ident.span(),
            Token::Literal(literal) => literal.span(),
            Token::Punct1(_, punct) => punct.span(),
            // FIXME join these spans one day
            Token::Punct2(_, _, punct, _) => punct.span(),
            Token::Punct3(_, _, _, punct, _, _) => punct.span(),
            Token::Group(_, group) => group.span(),
            Token::Keyword(_, ident) => ident.span(),
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
            Token::Punct1(_, punct) => punct.fmt(f),
            Token::Punct2(_, _, punct1, punct2) => {
                punct1.fmt(f)?;
                punct2.fmt(f)
            }
            Token::Punct3(_, _, _, punct1, punct2, punct3) => {
                punct1.fmt(f)?;
                punct2.fmt(f)?;
                punct3.fmt(f)
            }
            Token::Group(_, group) => group.fmt(f),
            Token::Keyword(_, ident) => ident.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tokens(Vec<Token>);

impl Tokens {
    pub fn to_stream(&self) -> TokenStream {
        to_stream(self)
    }

    pub fn lexer(&self) -> Lexer<'_> {
        Lexer::new(self)
    }

    pub fn new() -> Self {
        Tokens(Vec::new())
    }

    pub fn split_off(&mut self, index: usize) -> Self {
        Tokens(self.0.split_off(index))
    }
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens::new()
    }
}

impl Deref for Tokens {
    type Target = [Token];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tokens {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Token> for Tokens {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Token>,
    {
        Tokens(iter.into_iter().collect())
    }
}

impl IntoIterator for Tokens {
    type Item = Token;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Tokens {
    type Item = &'a Token;
    type IntoIter = std::slice::Iter<'a, Token>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<Token> for Tokens {
    fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Token>,
    {
        self.0.extend(it.into_iter());
    }
}

impl Extend<Tokens> for Tokens {
    fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Tokens>,
    {
        for tokens in it {
            self.0.extend(tokens);
        }
    }
}

impl Add<Tokens> for Tokens {
    type Output = Tokens;
    fn add(mut self, other: Tokens) -> Self::Output {
        self.extend(other);
        self
    }
}

impl Add<Token> for Tokens {
    type Output = Tokens;
    fn add(mut self, other: Token) -> Self::Output {
        self.extend(std::iter::once(other));
        self
    }
}

impl Add<Token> for Token {
    type Output = Tokens;
    fn add(self, other: Token) -> Self::Output {
        Tokens(vec![self, other])
    }
}

impl Add<Tokens> for Token {
    type Output = Tokens;
    fn add(self, mut other: Tokens) -> Self::Output {
        other.0.insert(0, self);
        other
    }
}

impl Add<Option<Token>> for Tokens {
    type Output = Tokens;
    fn add(mut self, token: Option<Token>) -> Self::Output {
        self.extend(token.into_iter());
        self
    }
}

impl Add<Option<Tokens>> for Tokens {
    type Output = Tokens;
    fn add(mut self, tokens: Option<Tokens>) -> Self::Output {
        if let Some(tokens) = tokens {
            self.extend(tokens);
        }
        self
    }
}

impl Add<Option<Token>> for Token {
    type Output = Tokens;
    fn add(self, token: Option<Token>) -> Self::Output {
        let mut out: Self::Output = self.into();
        out.extend(token.into_iter());
        out
    }
}

impl Add<Vec<Tokens>> for Tokens {
    type Output = Tokens;
    fn add(mut self, tokens_list: Vec<Tokens>) -> Self::Output {
        self.extend(tokens_list);
        self
    }
}

impl Add<Option<Tokens>> for Token {
    type Output = Tokens;
    fn add(self, tokens: Option<Tokens>) -> Self::Output {
        let mut out: Self::Output = self.into();
        if let Some(tokens) = tokens {
            out.extend(tokens);
        }
        out
    }
}

impl Display for Tokens {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for token in &self.0 {
            token.fmt(f)?;
        }
        Ok(())
    }
}

impl From<Token> for Tokens {
    fn from(token: Token) -> Self {
        Tokens(vec![token])
    }
}

impl From<Token> for TokenStream {
    fn from(token: Token) -> Self {
        to_stream(&[token])
    }
}

impl From<Tokens> for TokenStream {
    fn from(tokens: Tokens) -> Self {
        tokens.to_stream()
    }
}

impl From<Ident> for Token {
    fn from(ident: Ident) -> Self {
        Token::Ident(ident)
    }
}

impl From<Ident> for Tokens {
    fn from(ident: Ident) -> Self {
        Token::Ident(ident).into()
    }
}

impl From<Literal> for Token {
    fn from(literal: Literal) -> Self {
        Token::Literal(literal)
    }
}

impl From<Literal> for Tokens {
    fn from(literal: Literal) -> Self {
        Token::Literal(literal).into()
    }
}

impl From<Punct> for Token {
    fn from(punct: Punct) -> Self {
        Token::Punct1(punct.as_char(), punct)
    }
}

impl From<Punct> for Tokens {
    fn from(punct: Punct) -> Self {
        Token::Punct1(punct.as_char(), punct).into()
    }
}

impl From<Group> for Token {
    fn from(group: Group) -> Self {
        Token::Group(group.delimiter(), group)
    }
}

impl From<Group> for Tokens {
    fn from(group: Group) -> Self {
        Token::Group(group.delimiter(), group).into()
    }
}

impl From<TokenStream> for Tokens {
    fn from(stream: TokenStream) -> Self {
        let mut vec = Vec::new();
        for tt in stream {
            match tt {
                TokenTree::Ident(ident) => vec.push(keywordise(ident.into())),
                TokenTree::Literal(literal) => vec.push(literal.into()),
                TokenTree::Punct(punct) => {
                    if let Some(prev) = vec.pop() {
                        let next = match prev {
                            Token::Punct1(prev_chr, prev_punct) => {
                                if prev_punct.spacing() == Spacing::Joint {
                                    Token::Punct2(prev_chr, punct.as_char(), prev_punct, punct)
                                } else {
                                    vec.push(Token::Punct1(prev_chr, prev_punct));
                                    punct.into()
                                }
                            }
                            Token::Punct2(prev_chr_1, prev_chr_2, prev_p1, prev_p2) => {
                                if prev_p2.spacing() == Spacing::Joint {
                                    Token::Punct3(
                                        prev_chr_1,
                                        prev_chr_2,
                                        punct.as_char(),
                                        prev_p1,
                                        prev_p2,
                                        punct,
                                    )
                                } else {
                                    vec.push(Token::Punct2(
                                        prev_chr_1, prev_chr_2, prev_p1, prev_p2,
                                    ));
                                    punct.into()
                                }
                            }
                            prev => {
                                vec.push(prev);
                                punct.into()
                            }
                        };
                        vec.push(next);
                    } else {
                        vec.push(punct.into());
                    }
                }
                TokenTree::Group(group) => vec.push(group.into()),
            }
        }
        Tokens(vec)
    }
}

impl From<proc_macro::TokenStream> for Tokens {
    fn from(stream: proc_macro::TokenStream) -> Self {
        let stream: TokenStream = stream.into();
        stream.into()
    }
}

impl From<Tokens> for proc_macro::TokenStream {
    fn from(tokens: Tokens) -> Self {
        tokens.to_stream().into()
    }
}

pub fn to_stream<'a, I: IntoIterator<Item = &'a Token>>(tokens: I) -> TokenStream {
    let mut stream = TokenStream::new();
    for token in tokens {
        match token {
            Token::Ident(ident) => stream.extend(vec![TokenTree::Ident(ident.clone())]),
            Token::Literal(literal) => stream.extend(vec![TokenTree::Literal(literal.clone())]),
            Token::Punct1(_, punct) => stream.extend(vec![TokenTree::Punct(punct.clone())]),
            Token::Punct2(_, _, p1, p2) => stream.extend(vec![
                TokenTree::Punct(p1.clone()),
                TokenTree::Punct(p2.clone()),
            ]),
            Token::Punct3(_, _, _, p1, p2, p3) => stream.extend(vec![
                TokenTree::Punct(p1.clone()),
                TokenTree::Punct(p2.clone()),
                TokenTree::Punct(p3.clone()),
            ]),
            Token::Group(_, group) => stream.extend(vec![TokenTree::Group(group.clone())]),
            Token::Keyword(_, ident) => stream.extend(vec![TokenTree::Ident(ident.clone())]),
        }
    }
    stream
}

pub struct Lexer<'a> {
    stream: &'a [Token],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(stream: &'a [Token]) -> Self {
        Lexer { stream, pos: 0 }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Spanned<Token, usize, RsxParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.get(self.pos) {
            None => None,
            Some(token) => {
                self.pos += 1;
                Some(Ok((self.pos - 1, token.clone(), self.pos)))
            }
        }
    }
}
