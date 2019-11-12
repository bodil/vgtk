use crate::Component;

pub(crate) mod component;
mod gobject;
mod handler;
mod property;

pub use component::{PropTransform, VComponent};
pub use gobject::VObject;
pub use handler::VHandler;
pub use property::VProperty;

/// A node in the virtual component tree representing a `Component` or a Gtk widget.
///
/// Don't attempt to construct these directly. Use the `gtk!` macro instead.
pub enum VNode<Model: Component> {
    Object(VObject<Model>),
    Component(VComponent<Model>),
}

impl<Model: Component> VNode<Model> {
    pub(crate) fn get_child_props(&self) -> &[VProperty] {
        match self {
            VNode::Object(object) => &object.child_props,
            VNode::Component(comp) => &comp.child_props,
        }
    }

    pub(crate) fn get_child_prop(&self, name: &str) -> Option<&VProperty> {
        let props = self.get_child_props();
        for prop in props {
            if prop.name == name {
                return Some(prop);
            }
        }
        None
    }
}

/// An iterator over zero or one `VNode`s.
///
/// A `VNode` implements `IntoIterator` for this, so you can return a single
/// `VNode` in a code block in the `gtk!` macro without needing to convert it.
///
/// If you need to return an empty list of `VNode`s, use `VNode::empty()`.
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
    /// Make an empty iterator of `VNode`s.
    ///
    /// Use this inside a code block in the `gtk!` macro to return an empty list
    /// of child nodes.
    pub fn empty() -> VNodeIterator<Model> {
        VNodeIterator { node: None }
    }
}
