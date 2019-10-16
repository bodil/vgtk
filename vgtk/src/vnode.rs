use std::borrow::Borrow;
use std::rc::Rc;

use glib::{signal::SignalHandlerId, GString, Object, Type};

pub use crate::vcomp::VComponent;
use crate::{Component, Scope};

pub enum VNode<Model: Component> {
    Widget(VWidget<Model>),
    Component(VComponent<Model>),
}

impl<Model: Component> IntoIterator for VNode<Model> {
    type Item = VNode<Model>;
    type IntoIter = std::iter::Once<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

pub struct VWidget<Model: Component> {
    pub object_type: Type,
    pub properties: Vec<VProperty>,
    pub handlers: Vec<VHandler<Model>>,
    pub children: Vec<VNode<Model>>,
}

#[derive(Clone)]
pub struct VProperty {
    pub name: &'static str,
    pub set: Rc<dyn Fn(&Object, bool) + 'static>,
}

impl<Model: Component> VWidget<Model> {
    pub fn get_prop<S: Borrow<str>>(&self, name: S) -> Option<&VProperty> {
        let name = name.borrow();
        for prop in &self.properties {
            if prop.name == name {
                return Some(prop);
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct VHandler<Model: Component> {
    pub name: &'static str,
    pub id: &'static str,
    pub set: Rc<dyn Fn(&Object, &Scope<Model>) -> SignalHandlerId>,
}

pub trait PropertyCompare<'a, A> {
    fn property_compare(&self, other: &A) -> bool;
    fn property_convert(value: &'a A) -> Self;
}

impl<'a, A> PropertyCompare<'a, A> for A
where
    A: Eq + ToOwned<Owned = A>,
{
    fn property_compare(&self, other: &A) -> bool {
        self == other
    }

    fn property_convert(value: &A) -> Self {
        value.to_owned()
    }
}

impl<'a> PropertyCompare<'a, String> for &'a str {
    fn property_compare(&self, other: &String) -> bool {
        self == other
    }

    fn property_convert(value: &'a String) -> Self {
        value.as_str()
    }
}

impl<'a> PropertyCompare<'a, &'a String> for &'a str {
    fn property_compare(&self, other: &&String) -> bool {
        self == other
    }

    fn property_convert(value: &&'a String) -> Self {
        value.as_str()
    }
}

impl<'a> PropertyCompare<'a, &'a str> for Option<&'a str> {
    fn property_compare(&self, other: &&str) -> bool {
        match self {
            Some(ref value) => value == other,
            None => false,
        }
    }

    fn property_convert(value: &&'a str) -> Self {
        Some(*value)
    }
}

impl<'a> PropertyCompare<'a, &'a str> for GString {
    fn property_compare(&self, other: &&str) -> bool {
        self.as_str() == *other
    }

    fn property_convert(value: &&'a str) -> Self {
        (*value).into()
    }
}

impl<'a> PropertyCompare<'a, &'a str> for Option<GString> {
    fn property_compare(&self, other: &&str) -> bool {
        match self {
            Some(ref value) => value.as_str() == *other,
            None => false,
        }
    }

    fn property_convert(value: &&'a str) -> Self {
        Some((*value).into())
    }
}

impl<'a> PropertyCompare<'a, String> for Option<GString> {
    fn property_compare(&self, other: &String) -> bool {
        match self {
            Some(ref value) => value.as_str() == *other,
            None => false,
        }
    }

    fn property_convert(value: &'a String) -> Self {
        Some(value.as_str().into())
    }
}

impl<'a> PropertyCompare<'a, &'a String> for Option<GString> {
    fn property_compare(&self, other: &&String) -> bool {
        match self {
            Some(ref value) => value.as_str() == *other,
            None => false,
        }
    }

    fn property_convert(value: &&'a String) -> Self {
        Some(value.as_str().into())
    }
}
