use std::borrow::Borrow;

use glib::{Object, Type};

use super::{VHandler, VNode, VProperty};
use crate::Component;

pub struct VObject<Model: Component> {
    pub object_type: Type,
    pub constructor: Option<Box<dyn Fn() -> Object>>,
    pub properties: Vec<VProperty>,
    pub child_props: Vec<VProperty>,
    pub handlers: Vec<VHandler<Model>>,
    pub children: Vec<VNode<Model>>,
}

impl<Model: Component> VObject<Model> {
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
