use glib::prelude::*;
use glib::{Object, Type, Value};
use gtk::prelude::*;
use gtk::{Builder, Container, Widget};

use std::collections::BTreeMap as OrdMap;
use std::rc::Rc;

use component::{Component, Scope};
use event::SignalHandler;
use ffi;
use vobject::VObject;

fn build_obj<A: IsA<Object>>(class: Type, id: Option<&str>) -> A {
    let mut ui = String::new();
    ui += &format!("<interface><object class=\"{}\"", class);
    if let Some(id) = id {
        // TODO escape this string y'all
        ui += &format!(" id=\"{}\"", id);
    }
    ui += "/></interface>";

    let builder = Builder::new_from_string(&ui);
    let objects = builder.get_objects();
    objects
        .last()
        .unwrap_or_else(|| panic!("unknown class {}", class))
        .clone()
        .downcast::<A>()
        .unwrap_or_else(|_| panic!("build_obj: cannot cast {} to {}", class, A::static_type()))
}

pub struct GtkState<Model: Component> {
    object: Widget,
    props: OrdMap<String, Value>,
    handlers: OrdMap<String, Vec<Rc<SignalHandler<Model>>>>,
    children: Vec<GtkState<Model>>,
}

impl<Model: 'static + Component> GtkState<Model> {
    pub fn build(vobj: &VObject<Model>, parent: Option<&Container>, scope: &Scope<Model>) -> Self {
        let id = vobj
            .properties
            .get("id")
            .map(|v| v.get::<String>().expect("id property is not a string"));

        // Build this object
        let object = build_obj::<Widget>(vobj.type_, id.as_ref().map(String::as_str));

        // Apply properties
        for (prop, value) in &vobj.properties {
            if prop != "id" {
                // Attempt to set a property on the current object
                if let Err(e) = object.set_property(prop.as_str(), value) {
                    // If that fails, try it as a child property of the parent, if we have one
                    if let Some(parent) = parent {
                        ffi::set_child_property(parent, &object, prop.as_str(), value)
                            .unwrap_or_else(|e| {
                                panic!("invalid property {:?} for {}: {:?}", prop, vobj.type_, e)
                            });
                    } else {
                        panic!("invalid property {:?} for {}: {:?}", prop, vobj.type_, e)
                    }
                }
            }
        }

        // Apply handlers
        for (signal, handlers) in &vobj.handlers {
            for handler in handlers {
                handler.connect(signal.as_str(), object.upcast_ref(), scope.clone());
            }
        }

        let mut state = GtkState {
            object: object.clone(),
            props: vobj.properties.clone(),
            handlers: vobj.handlers.clone(),
            children: Vec::new(),
        };

        // Build children
        if !vobj.children.is_empty() {
            if let Some(parent) = object.downcast_ref::<Container>() {
                for child_spec in &vobj.children {
                    let child = Self::build(child_spec, Some(parent), scope);
                    {
                        parent.add(&child.object);
                    }
                    state.children.push(child);
                }
            } else {
                panic!("non-Container cannot have children: {}", vobj.type_);
            }
        }

        state
    }

    pub fn patch(
        &mut self,
        vobj: &VObject<Model>,
        parent: Option<&Container>,
        scope: &Scope<Model>,
    ) {
        // Patch children
        if let Some(parent) = self.object.downcast_ref::<Container>() {
            let mut to_remove = Vec::new();
            let mut to_append = Vec::new();
            let mut reconstruct_from = None;
            for index in 0..(self.children.len().max(vobj.children.len())) {
                match (self.children.get_mut(index), vobj.children.get(index)) {
                    (Some(target), Some(spec)) => {
                        if target.object.get_type() == spec.type_ {
                            // Objects have same type; patch down
                            target.patch(spec, Some(parent), scope);
                        } else {
                            // Objects are different, need to reconstruct everything from here
                            reconstruct_from = Some(index);
                            break;
                        }
                    }
                    (Some(_), None) => {
                        // Extraneous Gtk object; delete
                        to_remove.push(index);
                    }
                    (None, Some(spec)) => {
                        // New spec; construct
                        let state = Self::build(spec, Some(parent), scope);
                        parent.add(&state.object);
                        to_append.push(state);
                    }
                    (None, None) => break,
                }
            }
            if let Some(index) = reconstruct_from {
                // Remove all previous children from here onwards
                for child in self.children.drain(index..) {
                    parent.remove(&child.object);
                }
                // Rebuild children from new specs
                for child_spec in vobj.children.iter().skip(index) {
                    let state = Self::build(child_spec, Some(parent), scope);
                    parent.add(&state.object);
                    self.children.push(state);
                }
            } else {
                // Remove children flagged as extraneous
                for index in to_remove {
                    let child = self.children.remove(index);
                    parent.remove(&child.object);
                }
                // Or append newly constructed children
                self.children.extend(to_append);
            }
        }

        // Patch properties
        // TODO figure out how to avoid patching props which haven't changed
        for (prop, value) in &vobj.properties {
            if prop != "id" {
                // Attempt to set a property on the current object
                if let Err(e) = self.object.set_property(prop.as_str(), value) {
                    // If that fails, try it as a child property of the parent, if we have one
                    if let Some(parent) = parent {
                        ffi::set_child_property(parent, &self.object, prop.as_str(), value)
                            .unwrap_or_else(|e| {
                                panic!("invalid property {:?} for {}: {:?}", prop, vobj.type_, e)
                            });
                    } else {
                        panic!("invalid property {:?} for {}: {:?}", prop, vobj.type_, e)
                    }
                }
            }
        }

        // BIG TODO Patch handlers
    }

    pub fn object<O: IsA<Widget>>(&self) -> Option<&O> {
        self.object.downcast_ref()
    }
}
