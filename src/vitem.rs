use std::fmt::{Debug, Error, Formatter};

use component::Component;
use vcomp::VComponent;
use vobject::VObject;

pub enum VItem<Model: Component> {
    Component(VComponent<Model>),
    Object(VObject<Model>),
}

impl<Model: Component> Debug for VItem<Model> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            VItem::Component(_) => write!(f, "Component"),
            VItem::Object(obj) => obj.fmt(f),
        }
    }
}
