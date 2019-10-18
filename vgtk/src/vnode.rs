use std::borrow::Borrow;

use glib::{signal::SignalHandlerId, Object, Type};
use gtk::Container;

pub use crate::vcomp::VComponent;
use crate::{Component, Scope};

pub enum VNode<Model: Component> {
    Widget(VWidget<Model>),
    Component(VComponent<Model>),
}

impl<Model: Component> VNode<Model> {
    pub fn get_child_props(&self) -> &[VProperty] {
        match self {
            VNode::Widget(widget) => &widget.child_props,
            VNode::Component(comp) => &comp.child_props,
        }
    }

    pub fn get_child_prop(&self, name: &str) -> Option<&VProperty> {
        let props = self.get_child_props();
        for prop in props {
            if prop.name == name {
                return Some(prop);
            }
        }
        None
    }
}

pub struct VWidget<Model: Component> {
    pub object_type: Type,
    pub properties: Vec<VProperty>,
    pub child_props: Vec<VProperty>,
    pub handlers: Vec<VHandler<Model>>,
    pub children: Vec<VNode<Model>>,
}

pub struct VProperty {
    pub name: &'static str,
    pub set: Box<dyn Fn(&Object, Option<&Container>, bool) + 'static>,
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

pub struct VHandler<Model: Component> {
    pub name: &'static str,
    pub id: &'static str,
    pub set: Box<dyn Fn(&Object, &Scope<Model>) -> SignalHandlerId>,
}

pub struct VNodeIterator<Model: Component> {
    node: Option<VNode<Model>>,
}

impl<Model: Component> Iterator for VNodeIterator<Model> {
    type Item = VNode<Model>;
    fn next(&mut self) -> Option<Self::Item> {
        self.node.take()
    }
}

impl<Model: Component> IntoIterator for VNode<Model> {
    type Item = VNode<Model>;
    type IntoIter = VNodeIterator<Model>;
    fn into_iter(self) -> Self::IntoIter {
        VNodeIterator { node: Some(self) }
    }
}

impl<Model: Component> VNode<Model> {
    pub fn empty() -> VNodeIterator<Model> {
        VNodeIterator { node: None }
    }
}
