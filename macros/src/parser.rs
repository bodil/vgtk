use proc_macro2::{Group, Ident, Literal, Punct, Span};

use crate::combo::*;
use crate::context::{Attribute, GtkComponent, GtkElement, GtkWidget};
use crate::lexer::Token;

// fn spanned_fold<A, F, I>(iter: I, initial: A, f: F) -> (Span, A)
// where
//     I: IntoIterator<Item = Token>,
//     F: Fn(A, Token) -> A,
// {
//     let (span, result) = iter.into_iter().fold((None, initial), |(span, acc), next| {
//         (
//             match span {
//                 None => Some(next.span()),
//                 Some(span) => {
//                     #[cfg(can_join_spans)]
//                     {
//                         span.join(next.span())
//                     }
//                     #[cfg(not(can_join_spans))]
//                     {
//                         Some(span)
//                     }
//                 }
//             },
//             f(acc, next),
//         )
//     });
//     (span.unwrap_or_else(|| Span::call_site()), result)
// }

pub fn match_ident<'a>(name: &str) -> impl Parser<'a, Token, Token> {
    let name = name.to_string();
    move |input: &Cursor<'a, Token>| {
        let target = Ident::new(
            &name,
            input
                .get()
                .map(|i| i.span())
                .unwrap_or_else(|| Span::call_site()),
        );
        match input.get() {
            Some(Token::Ident(ident)) if ident == &target => {
                ok(ident.clone().into(), input, input.next())
            }
            _ => Err(Error::new(input).expect(format!("\"{}\"", target))),
        }
    }
}

pub fn ident<'a>() -> impl Parser<'a, Token, Ident> {
    |input: &Cursor<'a, Token>| match input.get() {
        Some(Token::Ident(ident)) => ok(ident.clone(), input, input.next()),
        _ => err(input, "an identifier"),
    }
}

pub fn literal<'a>() -> impl Parser<'a, Token, Literal> {
    |input: &Cursor<'a, Token>| match input.get() {
        Some(Token::Literal(literal)) => ok(literal.clone(), input, input.next()),
        _ => err(input, "a literal"),
    }
}

pub fn punct<'a>(name: char) -> impl Parser<'a, Token, Punct> {
    move |input: &Cursor<'a, Token>| match input.get() {
        Some(Token::Punct(punct, token)) if punct == &name => {
            ok(token.clone(), input, input.next())
        }
        _ => err(input, format!("\"{}\"", name)),
    }
}

pub fn not_punct<'a>(name: char) -> impl Parser<'a, Token, Token> {
    move |input: &Cursor<'a, Token>| match input.get() {
        Some(Token::Punct(punct, _)) if punct == &name => {
            err(input, format!("anything but \"{}\"", name))
        }
        Some(token) => ok(token.clone(), input, input.next()),
        None => err(input, format!("anything but \"{}\" or EOF", name)),
    }
}

pub fn group<'a>() -> impl Parser<'a, Token, Group> {
    |input: &Cursor<'a, Token>| match input.get() {
        Some(Token::Group(_, group)) => ok(group.clone(), input, input.next()),
        _ => err(input, "a code block"),
    }
}

pub fn rust_expr<'a>() -> impl Parser<'a, Token, Vec<Token>> {
    // TODO: currently only accepts literals, identifiers and blocks. JSX only
    // accepts strings and blocks, but I feel we could make more of an effort to
    // parse bare Rust expressions, eg. function/method calls.
    literal().map(|l| vec![l.into()])
        | group().map(|g| vec![g.into()])
        | ident().map(|i| vec![i.into()])
}

pub fn closure_args<'a>() -> impl Parser<'a, Token, Vec<Token>> {
    punct('|').right(expect(any(not_punct('|')).left(punct('|'))))
}

pub fn closure<'a>() -> impl Parser<'a, Token, (Vec<Token>, Vec<Token>)> {
    closure_args().pair(expect(rust_expr()))
}

pub fn property_attr<'a>() -> impl Parser<'a, Token, Attribute> {
    ident()
        .pair(punct('=').right(expect(rust_expr())))
        .map(|(name, value)| Attribute::Property { name, value })
}

pub fn handler_attr<'a>() -> impl Parser<'a, Token, Attribute> {
    match_ident("on").right(expect(
        ident()
            .pair(punct('=').right(closure()))
            .map(|(name, (args, body))| Attribute::Handler { name, args, body }),
    ))
}

pub fn attribute<'a>() -> impl Parser<'a, Token, Attribute> {
    handler_attr().or(property_attr())
}

pub fn end_tag<'a>(expected: Ident) -> impl Parser<'a, Token, Ident> {
    punct('<').right(
        punct('/').right(
            ident()
                .assert(move |ident| {
                    let expected = expected.to_string();
                    if ident.to_string() == expected {
                        Ok(ident)
                    } else {
                        Err(|err: Error<'a, Token>| {
                            err.expect(expected).describe("unexpected end tag")
                        })
                    }
                })
                .left(punct('>'))
                .expect(),
        ),
    )
}

pub fn widget<'a>() -> impl Parser<'a, Token, GtkWidget> {
    let open = punct('<').right(ident().pair(attribute().repeat(0..)).expect());
    open.and_then(move |(name, attributes)| {
        let tag_name = name.clone();
        let name2 = name.clone();
        let attrs2 = attributes.clone();
        punct('/')
            .left(punct('>').expect())
            .map(move |_| GtkWidget {
                name: name.clone(),
                attributes: attributes.clone(),
                children: Vec::new(),
            })
            .or(
                punct('>').right(element().to_box().repeat(0..).left(end_tag(tag_name)).map(
                    move |children| GtkWidget {
                        name: name2.clone(),
                        attributes: attrs2.clone(),
                        children,
                    },
                )),
            )
    })
}

pub fn component<'a>() -> impl Parser<'a, Token, GtkComponent> {
    let open =
        punct('<').right(punct('@').right(ident().pair(property_attr().repeat(0..)).expect()));
    open.and_then(move |(name, attributes)| {
        punct('/')
            .left(punct('>').expect())
            .map(move |_| GtkComponent {
                name: name.clone(),
                attributes: attributes.clone(),
            })
    })
}

pub fn element<'a>() -> impl Parser<'a, Token, GtkElement> {
    component().map(|component| GtkElement::Component(component))
        | widget().map(|widget| GtkElement::Widget(widget))
        | group().map(|group| GtkElement::Block(group))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::unroll_stream;
    use quote::quote;

    fn assert_widget<'a>(
        (name, attrs): (&str, &[(&str, &str)]),
        el: &'a GtkElement,
    ) -> &'a GtkWidget {
        match el {
            GtkElement::Widget(el) => {
                assert_eq!(name, &el.name.to_string());
                for (attr, expect) in el.attributes.iter().zip(attrs.iter()) {
                    assert_eq!(attr, expect);
                }
                el
            }
            _ => panic!("expected GtkWidget"),
        }
    }

    #[test]
    fn parse_elements() {
        let stream = unroll_stream(
            quote!(
                <Window title="title" width=500>
                    <Button label="wibble" />
                    <Button on click=|_| {panic!()} label="wobble" />
                    <Label label={omg.lol()} />
                </Window>
            ),
            false,
        );
        match element().parse(&stream.cursor()) {
            Ok(Success { value: window, .. }) => {
                let window = assert_widget(
                    ("Window", &[("title", "\"title\""), ("width", "500")]),
                    &window,
                );
                let mut children = window.children.iter();
                let button1 = children.next().unwrap();
                let button1 = assert_widget(("Button", &[("label", "\"wibble\"")]), button1);
                assert!(button1.children.is_empty());
                let button2 = children.next().unwrap();
                let button2 = assert_widget(
                    (
                        "Button",
                        &[("on click", "{| _ | panic ! ( )}"), ("label", "\"wobble\"")],
                    ),
                    button2,
                );
                assert!(button2.children.is_empty());
                let label = children.next().unwrap();
                let label = assert_widget(("Label", &[("label", "{omg . lol ( )}")]), label);
                assert!(label.children.is_empty());
                assert!(children.next().is_none());
            }
            Err(err) => panic!("failed to parse: {:?}", err),
        };
    }

    #[test]
    fn wrong_closing_tag() {
        let stream = unroll_stream(
            quote!(
                <Window title="title" width=500>
                    <Button onlabel="wibble" />
                    <Button label="wobble" />
                    <Label label={omg.lol()}/>
                </Label>
            ),
            false,
        );
        match element().parse(&stream.cursor()) {
            Ok(_) => panic!("successfully parsed an invalid tag"),
            Err(err) => {
                assert_eq!(Some("unexpected end tag".to_string()), err.description);
                assert_eq!(true, err.fatal);
                let mut expected = std::collections::BTreeSet::new();
                expected.insert("Window".to_string());
                assert_eq!(expected, err.expected);
                assert_eq!("Label", err.input.get().unwrap().to_string());
            }
        };
    }
}
