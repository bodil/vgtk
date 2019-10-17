use std::fmt::{Debug, Error, Formatter};

use proc_macro2::{Group, Ident};

use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct GtkWidget {
    pub name: Ident,
    pub attributes: Vec<Attribute>,
    pub children: Vec<GtkElement>,
}

#[derive(Debug, Clone)]
pub struct GtkComponent {
    pub name: Vec<Token>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum GtkElement {
    Widget(GtkWidget),
    Component(GtkComponent),
    Block(Group),
}

#[derive(Clone)]
pub enum Attribute {
    Property {
        parent: Vec<Token>,
        name: Ident,
        value: Vec<Token>,
    },
    Handler {
        name: Ident,
        args: Vec<Token>,
        body: Vec<Token>,
    },
}

fn stringify_attr_value(token: &Token) -> String {
    match token {
        Token::Ident(l) => l.to_string(),
        Token::Literal(l) => l.to_string(),
        Token::Group(_, g) => g.to_string(),
        _ => unimplemented!(),
    }
}

impl Debug for Attribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Attribute::Property {
                parent,
                name,
                value,
            } => {
                let attrs: Vec<String> = value.iter().map(stringify_attr_value).collect();
                write!(
                    f,
                    "( {}{} = {} )",
                    parent
                        .iter()
                        .map(|p| format!("{}::", p))
                        .collect::<String>(),
                    name.to_string(),
                    attrs.join(", ")
                )
            }
            Attribute::Handler { name, args, body } => {
                let args: Vec<String> = args.iter().map(stringify_attr_value).collect();
                let attrs: Vec<String> = body.iter().map(stringify_attr_value).collect();
                write!(
                    f,
                    "( {} = {} {} )",
                    name.to_string(),
                    args.join(", "),
                    attrs.join(", ")
                )
            }
        }
    }
}

impl PartialEq<(&str, &str)> for Attribute {
    fn eq(&self, other: &(&str, &str)) -> bool {
        match self {
            Attribute::Property {
                parent,
                name,
                value,
            } => {
                let name = format!(
                    "{}{}",
                    parent
                        .iter()
                        .map(|p| format!("{}::", p))
                        .collect::<String>(),
                    name.to_string()
                );
                name == other.0 && stringify_attr_value(&value[0]) == other.1
            }
            Attribute::Handler { name, .. } => {
                format!("on {}", name.to_string()) == other.0 // FIXME: only compares handler name
            }
        }
    }
}
