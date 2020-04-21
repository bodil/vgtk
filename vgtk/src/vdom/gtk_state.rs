use std::collections::{HashMap, HashSet};

use gio::{Action, ActionExt, ActionMapExt};
use glib::{prelude::*, Object, SignalHandlerId};
use gtk::{
    self, prelude::*, Application, ApplicationWindow, Bin, Box as GtkBox, Builder, Container,
    Dialog, Grid, GridExt, Menu, MenuButton, MenuItem, ShortcutsWindow, Widget, Window,
    HeaderBar
};

use super::State;
use crate::component::Component;
use crate::scope::Scope;
use crate::vnode::{VHandler, VNode, VObject, VProperty};

pub(crate) struct GtkState<Model: Component> {
    pub(crate) object: Object,
    handlers: HashMap<(&'static str, &'static str), SignalHandlerId>,
    children: Vec<State<Model>>,
}

fn build_obj<A: IsA<Object>, Model: Component>(spec: &VObject<Model>) -> A {
    let class = spec.object_type;
    let obj = if let Some(ref cons) = spec.constructor {
        cons()
    } else {
        let mut ui = String::new();
        ui += &format!("<interface><object class=\"{}\"", class);
        ui += "/></interface>";

        let builder = Builder::new_from_string(&ui);
        let objects = builder.get_objects();
        objects
            .last()
            .unwrap_or_else(|| panic!("unknown class {}", class))
            .clone()
    };
    obj.downcast::<A>()
        .unwrap_or_else(|_| panic!("build_obj: cannot cast {} to {}", class, A::static_type()))
}

// Gtk has many strange ways of adding children to a parent.
fn add_child<Model: Component>(
    parent: &Object,
    index: usize,
    total: usize,
    child_spec: &VNode<Model>,
    child: &Object,
) {
    if let Some(application) = parent.downcast_ref::<Application>() {
        if let Some(window) = child.downcast_ref::<Window>() {
            application.add_window(window);
        } else if let Some(action) = child.downcast_ref::<Action>() {
            application.add_action(action);
        } else {
            panic!(
                "Application's children must be Windows or Actions, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(button) = parent.downcast_ref::<MenuButton>() {
        // MenuButton: can only have a single child, either a `Menu` set with
        // `set_popup` or any other `Widget` set with `set_popover`.
        if total > 1 {
            panic!(
                "MenuButton can only have 1 child, but {} were found.",
                total,
            );
        }
        if let Some(menu) = child.downcast_ref::<Menu>() {
            button.set_popup(Some(menu));
        } else if let Some(widget) = child.downcast_ref::<Widget>() {
            button.set_popover(Some(widget));
        } else {
            panic!(
                "MenuButton's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(item) = parent.downcast_ref::<MenuItem>() {
        // MenuItem: single child, must be a `Menu`, set with `set_submenu`.
        if total > 1 {
            panic!("MenuItem can only have 1 child, but {} were found.", total);
        }
        if let Some(menu) = child.downcast_ref::<Menu>() {
            item.set_submenu(Some(menu));
        } else {
            panic!(
                "MenuItem can only take children of type Menu, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(dialog) = parent.downcast_ref::<Dialog>() {
        // Dialog: children must be added to the Dialog's content area through
        // get_content_area().
        if let Some(widget) = child.downcast_ref::<Widget>() {
            dialog.get_content_area().add(widget);
        } else {
            panic!(
                "Dialog's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(window) = parent.downcast_ref::<ApplicationWindow>() {
        // ApplicationWindow: takes any number of Actions, optionally one
        // ShortcutsWindow added with `set_help_overlay()`, and either 1 or 2
        // Widgets. If 1, it's the main widget. If 2, the first is added with
        // `set_titlebar()` and the second is the main widget.
        if let Some(action) = child.downcast_ref::<Action>() {
            window.add_action(action);
        } else if let Some(help_overlay) = child.downcast_ref::<ShortcutsWindow>() {
            window.set_help_overlay(Some(help_overlay));
        } else if let Some(widget) = child.downcast_ref::<Widget>() {
            match window.get_child() {
                None => window.add(widget),
                Some(ref titlebar) if window.get_titlebar().is_none() => {
                    window.remove(titlebar);
                    window.set_titlebar(Some(titlebar));
                    window.add(widget);
                }
                _ => panic!("ApplicationWindow can have at most two Widget children."),
            }
        } else {
            panic!(
                "ApplicationWindow's children must be Actions or Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(window) = parent.downcast_ref::<Window>() {
        // Window: takes only 1 or 2 Widgets. If 1 widget child, it's the
        // window's main widget. If 2, the first is the title bar and the second
        // is the main widget. More than 2 goes boom.
        if let Some(widget) = child.downcast_ref::<Widget>() {
            if total == 2 && index == 0 {
                window.set_titlebar(Some(widget));
            } else {
                window.add(widget);
            }
        } else {
            panic!(
                "Window's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(parent) = parent.downcast_ref::<Bin>() {
        // Bin: can only have a single child.
        if total > 1 {
            panic!("Bins can only have 1 child, but {} were found.", total);
        }
        if let Some(widget) = child.downcast_ref::<Widget>() {
            parent.add(widget);
        } else {
            panic!(
                "Bin's child must be a Widget, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(parent) = parent.downcast_ref::<GtkBox>() {
        // Box: added normally, except one widget can be added using
        // set_center_widget() if it has the center_widget=true child property
        // (which is faked in ext.rs). More than one child with this property is
        // undefined behaviour.
        if let Some(widget) = child.downcast_ref::<Widget>() {
            if child_spec.get_child_prop("center_widget").is_some() {
                parent.set_center_widget(Some(widget));
            } else {
                parent.add(widget);
            }
        } else {
            panic!(
                "Box's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(parent) = parent.downcast_ref::<HeaderBar>() {
        // HeaderBar: added normally, except one widget can be added using
        // set_custom_title if it has the custom_title=true child property
        // (which is faked in ext.rs). More than one child with this property is
        // undefined behaviour.
        if let Some(widget) = child.downcast_ref::<Widget>() {
            if child_spec.get_child_prop("custom_title").is_some() {
                parent.set_custom_title(Some(widget));
            } else {
                parent.add(widget);
            }
        } else {
            panic!(
                "HeaderBar's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(parent) = parent.downcast_ref::<Grid>() {
        if let Some(widget) = child.downcast_ref::<Widget>() {
            // by default we put widgets in the top left corner of the grid
            // with row and col span of 1; this would typically get overridden
            // via props but setting the default is important in order to avoid
            // making the user specify these for every single child widget
            parent.attach(widget, 0, 0, 1, 1);
        } else {
            panic!(
                "Grid's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else if let Some(container) = parent.downcast_ref::<Container>() {
        if let Some(widget) = child.downcast_ref::<Widget>() {
            container.add(widget);
        } else {
            panic!(
                "Container's children must be Widgets, but {} was found.",
                child.get_type()
            );
        }
    } else {
        panic!("Don't know how to add children to a {}", parent.get_type());
    }
    // Apply child properties
    for prop in child_spec.get_child_props() {
        (prop.set)(child.upcast_ref(), Some(parent), true);
    }
}

fn remove_child(parent: &Object, child: &Object) {
    // There are also special cases for removing children.
    if let Some(application) = parent.downcast_ref::<Application>() {
        if let Some(window) = child.downcast_ref::<Window>() {
            application.remove_window(window);
        } else if let Some(action) = child.downcast_ref::<Action>() {
            application.remove_action(&action.get_name().expect("Action unexpectedly has no name"));
        } else {
            panic!(
                "Applications can only contain Windows, but was asked to remove a {}.",
                child.get_type()
            );
        }
    } else if let Some(container) = parent.downcast_ref::<Container>() {
        // For a Container and a Widget child, we should always be able to call
        // `Container::remove`.
        if let Some(child_widget) = child.downcast_ref::<Widget>() {
            container.remove(child_widget);
        } else {
            panic!(
                "Containers can only contain Widgets but was asked to remove a {}.",
                child.get_type()
            );
        }
    } else {
        panic!(
            "Don't know how to remove a child from a {}",
            parent.get_type()
        );
    }
}

impl<Model: 'static + Component> GtkState<Model> {
    // This function build the root object, but not its children. You must call
    // `build_children()` to finalise construction.
    pub(crate) fn build_root(
        vobj: &VObject<Model>,
        parent: Option<&Object>,
        scope: &Scope<Model>,
    ) -> Self {
        // Build this object
        let object: Object = build_obj(&vobj);

        // Apply properties
        for prop in &vobj.properties {
            (prop.set)(object.upcast_ref(), parent, true);
        }

        // Apply handlers
        let mut handlers = HashMap::new();
        for handler in &vobj.handlers {
            let handle = (handler.set)(object.upcast_ref(), scope);
            handlers.insert((handler.name, handler.id), handle);
        }

        GtkState {
            object: object.upcast(),
            handlers,
            children: Vec::new(),
        }
    }

    pub(crate) fn build_children(&mut self, vobj: &VObject<Model>, scope: &Scope<Model>) {
        let object = &self.object;
        // Build children
        let total_children = vobj.children.len();
        for (index, child_spec) in vobj.children.iter().enumerate() {
            let child = State::build(child_spec, Some(&object), &scope);
            let child_object = child.object().clone();
            add_child(&object, index, total_children, child_spec, &child_object);
            self.children.push(child);
        }

        // Show this object, if it's a widget
        if let Some(widget) = self.object.downcast_ref::<Widget>() {
            widget.show();
        }
    }

    pub(crate) fn build(
        vobj: &VObject<Model>,
        parent: Option<&Object>,
        scope: &Scope<Model>,
    ) -> Self {
        let mut state = Self::build_root(vobj, parent, scope);
        state.build_children(vobj, scope);
        state
    }

    pub(crate) fn patch(
        &mut self,
        vobj: &VObject<Model>,
        parent: Option<&Object>,
        scope: &Scope<Model>,
    ) -> bool {
        // Patch children
        let mut to_remove = None;
        let mut to_append = Vec::new();
        let mut reconstruct_from = None;
        for index in 0..(self.children.len().max(vobj.children.len())) {
            match (self.children.get_mut(index), vobj.children.get(index)) {
                (Some(State::Component(target)), Some(spec_item)) => {
                    match spec_item {
                        VNode::Object(_) => {
                            // Component has become a widget; reconstruct from here
                            reconstruct_from = Some(index);
                            break;
                        }
                        VNode::Component(ref spec) => {
                            if !target.patch(spec, Some(&self.object), scope) {
                                reconstruct_from = Some(index);
                                break;
                            }
                        }
                    }
                }
                (Some(State::Gtk(target)), Some(spec_item)) => {
                    match spec_item {
                        VNode::Object(ref spec) => {
                            if target.object.get_type() == spec.object_type {
                                // Objects have same type; patch down
                                target.patch(spec, Some(&self.object), scope);
                            } else {
                                // Objects are different, need to reconstruct everything from here
                                reconstruct_from = Some(index);
                                break;
                            }
                        }
                        VNode::Component(_) => {
                            // Gtk object has turned into a component; reconstruct from here
                            reconstruct_from = Some(index);
                            break;
                        }
                    }
                }
                (Some(_), None) => {
                    // Extraneous Gtk object; delete
                    if to_remove.is_none() {
                        to_remove = Some(index);
                    }
                    break;
                }
                (None, Some(spec)) => {
                    // New spec; construct
                    let state = State::build(spec, Some(&self.object), scope);
                    add_child(
                        &self.object,
                        index,
                        vobj.children.len(),
                        spec,
                        state.object(),
                    );
                    to_append.push(state);
                }
                (None, None) => break,
            }
        }
        if let Some(index) = reconstruct_from {
            // Remove all previous children from here onwards
            if self.object.is::<Window>() && index == 0 && self.children.len() == 2 {
                panic!("Can't remove a title bar widget from an existing Window!");
            }
            for child in self.children.drain(index..) {
                remove_child(&self.object, child.object());
                child.unmount();
            }
            // Rebuild children from new specs
            for (index, child_spec) in vobj.children.iter().enumerate().skip(index) {
                let state = State::build(child_spec, Some(&self.object), scope);
                add_child(
                    &self.object,
                    index,
                    vobj.children.len(),
                    child_spec,
                    state.object(),
                );
                if let Some(w) = state.widget() {
                    w.show()
                }
                self.children.push(state);
            }
        } else {
            // Remove children flagged as extraneous
            if let Some(remove_from) = to_remove {
                if self.object.is::<Window>() && remove_from == 1 && self.children.len() == 2 {
                    panic!("Can't remove a title bar widget from an existing Window!");
                }
                for child in self.children.drain(remove_from..) {
                    remove_child(&self.object, &child.object());
                    child.unmount();
                }
            }
            // Or append newly constructed children
            if self.object.is::<Window>() && !to_append.is_empty() && self.children.len() == 1 {
                panic!("Can't add a title bar widget to an existing Window!");
            }
            for child in to_append {
                if let Some(w) = child.widget() {
                    w.show()
                }
                self.children.push(child);
            }
        }

        // Patch properties
        self.patch_properties(&vobj.properties, parent);

        // Patch child properties
        self.patch_properties(&vobj.child_props, parent);

        // Patch handlers
        self.patch_handlers(&vobj.handlers, scope);

        true
    }

    fn patch_properties(&mut self, properties: &[VProperty], parent: Option<&Object>) {
        for prop in properties {
            (prop.set)(self.object.upcast_ref(), parent, false);
        }
    }

    fn patch_handlers(&mut self, handlers: &[VHandler<Model>], scope: &Scope<Model>) {
        // FIXME need to store and match IDs
        let mut seen = HashSet::new();
        let mut remove = Vec::new();
        for handler in handlers {
            let key = (handler.name, handler.id);
            seen.insert(key.clone());
            if let std::collections::hash_map::Entry::Vacant(entry) = self.handlers.entry(key) {
                let handle = (handler.set)(self.object.upcast_ref(), scope);
                entry.insert(handle);
            }
        }
        for key in self.handlers.keys() {
            if !seen.contains(key) {
                remove.push(key.clone());
            }
        }
        for key in remove {
            let obj: &Object = self.object.upcast_ref();
            obj.disconnect(self.handlers.remove(&key).unwrap());
        }
    }

    pub(crate) fn unmount(self) {
        for child in self.children {
            child.unmount();
        }
        if let Ok(widget) = self.object.downcast::<Widget>() {
            widget.destroy();
        }
    }
}
