use std::fmt::{Debug, Error, Formatter};

use proc_macro2::{Group, Ident};

use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct GtkWidget {
    pub name: Ident,
    pub constructor: Option<Vec<Token>>,
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
        child: bool,
        parent: Vec<Token>,
        name: Ident,
        value: Vec<Token>,
    },
    Handler {
        name: Ident,
        async_keyword: Option<Token>,
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
                child,
                parent,
                name,
                value,
            } => {
                let attrs: Vec<String> = value.iter().map(stringify_attr_value).collect();
                let mut name = name.to_string();
                if !parent.is_empty() {
                    let parent_path: String = parent.iter().map(|p| format!("{}", p)).collect();
                    let qual = if *child { "" } else { "@" };
                    name = format!("{}{}{}", qual, parent_path, name);
                }
                write!(f, "( {} = {} )", name, attrs.join(", "))
            }
            Attribute::Handler {
                name,
                async_keyword,
                args,
                body,
            } => {
                let args: Vec<String> = args.iter().map(stringify_attr_value).collect();
                let attrs: Vec<String> = body.iter().map(stringify_attr_value).collect();
                let async_keyword = if async_keyword.is_some() {
                    "async "
                } else {
                    ""
                };
                write!(
                    f,
                    "( {} = {}{} {} )",
                    name.to_string(),
                    async_keyword,
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
                child,
                parent,
                name,
                value,
            } => {
                let mut name = name.to_string();
                if !parent.is_empty() {
                    let parent_path: String = parent.iter().map(|p| format!("{}", p)).collect();
                    let qual = if *child { "" } else { "@" };
                    name = format!("{}{}{}", qual, parent_path, name);
                }
                name == other.0 && stringify_attr_value(&value[0]) == other.1
            }
            Attribute::Handler { name, .. } => {
                format!("on {}", name.to_string()) == other.0 // FIXME: only compares handler name
            }
        }
    }
}
