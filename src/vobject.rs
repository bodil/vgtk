use glib::prelude::*;
use glib::{Object, Type, Value};
use gtk::prelude::*;

use std::collections::BTreeMap as OrdMap;
use std::fmt::{self, Debug};
use std::rc::Rc;

use component::Component;
use event::SignalHandler;

pub struct VObject<Model: Component> {
    pub type_: Type,
    pub properties: OrdMap<String, Value>,
    pub handlers: OrdMap<String, Vec<Rc<SignalHandler<Model>>>>,
    pub children: Vec<Rc<VObject<Model>>>,
}

impl<Model: Component> Default for VObject<Model> {
    fn default() -> Self {
        VObject {
            type_: Object::static_type(),
            properties: Default::default(),
            handlers: Default::default(),
            children: Default::default(),
        }
    }
}

impl<Model: Component + 'static> VObject<Model> {
    pub fn new(type_: Type) -> Self {
        VObject {
            type_,
            properties: Default::default(),
            handlers: Default::default(),
            children: Vec::new(),
        }
    }

    pub fn set_property<Prop, Val>(&mut self, prop: Prop, value: &Val)
    where
        Prop: Into<String>,
        Val: ToValue,
    {
        self.properties.insert(prop.into(), value.to_value());
    }

    pub fn add_handler<S: Into<String>>(&mut self, signal: S, handler: SignalHandler<Model>) {
        self.handlers
            .entry(signal.into())
            .or_default()
            .push(Rc::new(handler));
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(Rc::new(child))
    }
}

impl<Model: Component> Debug for VObject<Model> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.type_)
    }
}
