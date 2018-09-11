use glib::prelude::*;
use glib::{Object, Type, Value};
use gtk::prelude::*;
use gtk::{Bin, Builder, Container, Widget};

use std::collections::BTreeMap as OrdMap;
use std::fmt::{self, Debug};
use std::rc::Rc;

use component::{Component, Scope};
use event::SignalHandler;
use ffi;

fn build_obj(class: Type, id: Option<&str>) -> Object {
    let mut ui = String::new();
    ui += &format!("<interface><object class=\"{}\"", class);
    if let Some(id) = id {
        ui += &format!(" id=\"{}\"", id);
    }
    ui += "/></interface>";

    let builder = Builder::new_from_string(&ui);
    let objects = builder.get_objects();
    objects
        .last()
        .unwrap_or_else(|| panic!("unknown class {}", class))
        .clone()
}

fn normalise_prop(prop: &str) -> String {
    prop.replace('_', "-")
}

pub struct VObject<Model: Component> {
    pub type_: Type,
    properties: OrdMap<String, Value>,
    handlers: OrdMap<String, Vec<SignalHandler<Model>>>,
    children: Vec<Rc<VObject<Model>>>,
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
            .push(handler);
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(Rc::new(child))
    }

    fn construct(&self, scope: &Scope<Model>) -> (Object, OrdMap<String, Value>) {
        let mut extra_props = OrdMap::new();

        let id = self
            .properties
            .get("id")
            .map(|v| v.get::<String>().expect("id property is not a string!"));

        let obj = build_obj(self.type_, id.as_ref().map(String::as_str));

        for (prop, value) in &self.properties {
            if prop != "id" {
                let prop = normalise_prop(prop);
                if obj.set_property(prop.as_str(), value).is_err() {
                    extra_props.insert(prop, value.clone());
                }
            }
        }

        for (signal, handlers) in &self.handlers {
            for handler in handlers {
                handler.connect(signal.as_str(), &obj, scope.clone());
            }
        }

        if !self.children.is_empty() {
            if let Some(parent) = obj.downcast_ref::<Container>() {
                for child_spec in &self.children {
                    let (child_obj, extra_props) = child_spec.construct(scope);
                    let child: Widget = child_obj.downcast().expect("non-Widget child");
                    parent.add(&child);
                    for (prop, value) in extra_props {
                        ffi::set_child_property(&parent, &child, prop.as_str(), &value)
                            .unwrap_or_else(|e| {
                                panic!(
                                    "invalid property {:?} for {}: {:?}",
                                    prop, child_spec.type_, e
                                )
                            });
                    }
                }
            }
        }

        (obj, extra_props)
    }

    pub fn build<A>(&self, scope: &Scope<Model>) -> A
    where
        A: IsA<Object>,
    {
        let (obj, extra_props) = self.construct(scope);
        if !extra_props.is_empty() {
            panic!(
                "invalid properties {:?} for {}",
                extra_props.keys(),
                self.type_
            );
        }
        obj.downcast()
            .unwrap_or_else(|_| panic!("cannot downcast {} to return type", self.type_))
    }

    fn patch_obj(&self, scope: &Scope<Model>, target: &Object) {
        // TODO patch handlers
        // Patch children of a Container
        if let Some(parent) = target.downcast_ref::<Container>() {
            let mut children = parent.get_children();
            children.reverse(); // they appear to be in reverse order in there...
            let child_count = children.len();
            // If target has more children than desired, remove the surplus
            if child_count > self.children.len() {
                for child in children.split_off(self.children.len()) {
                    parent.remove(&child);
                }
            }
            // Patch each child in turn
            for (child, spec) in children.into_iter().zip(&self.children) {
                if child.get_type() == spec.type_ {
                    spec.patch_obj(scope, child.upcast_ref());
                } else {
                    // TODO this ought to be possible
                    panic!(
                        "child mismatch at {} => {} in {:?} => {:?}",
                        child.get_type(),
                        spec.type_,
                        parent
                            .get_children()
                            .iter()
                            .map(|c| c.get_type())
                            .collect::<Vec<_>>(),
                        self.children.iter().map(|c| c.type_).collect::<Vec<_>>()
                    );
                    // panic!("cannot change {} to {}", child.get_type(), spec.type_);
                }
            }
            // If spec has more children than target, construct the surplus
            if self.children.len() > child_count {
                for spec in self.children.iter().skip(child_count) {
                    let (child_obj, extra_props) = self.construct(scope);
                    let child: Widget = child_obj.downcast().unwrap();
                    parent.add(&child);
                    for (prop, value) in extra_props {
                        ffi::set_child_property(&parent, &child, prop.as_str(), &value)
                            .unwrap_or_else(|e| {
                                panic!("invalid property {:?} for {}: {:?}", prop, spec.type_, e)
                            });
                    }
                }
            }
        }
        // Patch child of a Bin
        else if let Some(parent) = target.downcast_ref::<Bin>() {
            let child = parent.get_child();
            debug_assert!(child.is_some());
            if self.children.len() != 1 {
                panic!(
                    "{} must have exactly one child, but has {}",
                    self.type_,
                    self.children.len()
                );
            }
            if let (Some(child), Some(spec)) = (child, self.children.first()) {
                if child.get_type() == spec.type_ {
                    spec.patch_obj(scope, child.upcast_ref());
                } else {
                    // TODO this ought to be possible
                    panic!("cannot change {} to {}", child.get_type(), spec.type_);
                }
            } else {
                unreachable!();
            }
        }
        self.patch_props(target);
    }

    fn patch_props(&self, target: &Object) -> OrdMap<String, Value> {
        let mut extra_props = OrdMap::new();
        for (prop, new_value) in &self.properties {
            if target.set_property(prop.as_str(), new_value).is_err() {
                extra_props.insert(prop.to_owned(), new_value.clone());
            }
        }
        extra_props
    }

    pub fn patch<A>(&self, scope: &Scope<Model>, target: &A)
    where
        A: Cast,
    {
        let root = target.upcast_ref();
        assert_eq!(
            self.type_,
            root.get_type(),
            "toplevel widget cannot change its type!"
        );
        self.patch_obj(scope, root)
    }
}

impl<Model: Component> Debug for VObject<Model> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.type_)
    }
}
