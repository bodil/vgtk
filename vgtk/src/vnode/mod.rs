use crate::Component;

pub(crate) mod component;
mod gobject;
mod handler;
mod property;

pub use component::{PropTransform, VComponent};
pub use gobject::VObject;
pub use handler::VHandler;
pub use property::VProperty;

pub enum VNode<Model: Component> {
    Object(VObject<Model>),
    Component(VComponent<Model>),
}

impl<Model: Component> VNode<Model> {
    pub fn get_child_props(&self) -> &[VProperty] {
        match self {
            VNode::Object(object) => &object.child_props,
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
