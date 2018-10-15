use glib::prelude::*;
use glib::{Object, Type, Value};

use std::fmt::{self, Debug};
use std::rc::Rc;

use im::{OrdMap, OrdSet};

use component::Component;
use event::SignalHandler;

pub struct VObject<Model: Component> {
    pub type_: Type,
    pub properties: OrdMap<String, Value>,
    pub handlers: OrdMap<String, OrdSet<Rc<SignalHandler<Model>>>>,
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

impl<Model: Component> VObject<Model> {
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
            .insert(Rc::new(handler));
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(Rc::new(child))
    }

    fn debug_print(&self, indent: usize, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{: >indent$}<{}", "", self.type_, indent = indent)?;
        for (prop, value) in &self.properties {
            write!(f, " {}={:?}", prop, value)?;
        }
        if self.children.is_empty() {
            write!(f, "/>")
        } else {
            write!(f, ">\n")?;
            for child in &self.children {
                child.debug_print(indent + 2, f)?;
                write!(f, "\n")?;
            }
            write!(f, "{: >indent$}</{}>", "", self.type_, indent = indent)
        }
    }
}

impl<Model: Component> Debug for VObject<Model> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.debug_print(0, f)
    }
}
