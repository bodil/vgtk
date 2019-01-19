use glib::prelude::*;
use glib::{Object, Type, Value};
use gtk::prelude::*;
use gtk::{self, Builder, Container, IconSize, Image, Widget, Window};

use std::any::TypeId;
use std::marker::PhantomData;
use std::rc::Rc;

use im::ordmap::DiffItem;
use im::{OrdMap, OrdSet};

use component::{Component, Scope, View};
use event::SignalHandler;
use ffi;
use vcomp::{unwrap_props, AnyProps};
use vitem::VItem;
use vobject::VObject;

pub enum State<Model: Component + View<Model>> {
    Gtk(GtkState<Model>),
    Component(ComponentState<Model>),
}

impl<Model: 'static + Component + View<Model>> State<Model> {
    pub fn build(vitem: &VItem<Model>, parent: Option<&Container>, scope: &Scope<Model>) -> Self {
        match vitem {
            VItem::Object(vobj) => State::Gtk(GtkState::build(vobj, parent, scope)),
            VItem::Component(vcomp) => State::Component((vcomp.constructor)(vcomp.props, parent)),
        }
    }

    pub fn patch(
        &mut self,
        vitem: &VItem<Model>,
        parent: Option<&Container>,
        scope: &Scope<Model>,
    ) {
        match vitem {
            VItem::Object(vobj) => match self {
                State::Gtk(state) => state.patch_object(vobj, parent, scope),
                State::Component(_state) => unimplemented!(),
            },
            VItem::Component(_) => unimplemented!(),
        }
    }

    pub fn object(&self) -> &Widget {
        match self {
            State::Gtk(state) => &state.object,
            State::Component(state) => &state.object,
        }
    }
}

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

trait PropertiesReceiver {
    fn update(&mut self, props: AnyProps);
}

pub struct ComponentState<Model: Component> {
    parent: PhantomData<Model>,
    object: Widget,
    model_type: TypeId,
    state: Box<dyn PropertiesReceiver>,
}

impl<Model: Component + View<Model>> ComponentState<Model> {
    pub fn new<Child: 'static + Component + View<Child>>(
        props: AnyProps,
        parent: Option<&Container>,
    ) -> Self {
        let (sub_state, object) = SubcomponentState::<Child>::new(props, parent);
        ComponentState {
            parent: PhantomData,
            object,
            model_type: TypeId::of::<Child>(),
            state: Box::new(sub_state),
        }
    }
}

pub struct SubcomponentState<Model: Component + View<Model>> {
    scope: Scope<Model>,
    state: Model,
    tree_state: State<Model>,
}

impl<Model: 'static + Component + View<Model>> SubcomponentState<Model> {
    fn new(props: AnyProps, parent: Option<&Container>) -> (Self, Widget) {
        let props: Model::Properties = unwrap_props(props);
        let scope = Scope::default();
        let state = Model::create(props);
        let tree = state.view();
        let tree_state = State::build(&tree, parent, &scope);
        let widget = tree_state.object().clone();
        (
            SubcomponentState {
                scope,
                state,
                tree_state,
            },
            widget,
        )
    }
}

impl<Model: Component + View<Model>> PropertiesReceiver for SubcomponentState<Model> {
    fn update(&mut self, _props: AnyProps) {}
}

pub struct GtkState<Model: Component + View<Model>> {
    object: Widget,
    props: OrdMap<String, Value>,
    handlers: OrdMap<String, OrdSet<Rc<SignalHandler<Model>>>>,
    children: Vec<State<Model>>,
}

fn eq_values(left: &Value, right: &Value) -> bool {
    if left.type_() != right.type_() {
        return false;
    }
    // This is painful, but what can you do
    format!("{:?}", left) == format!("{:?}", right)
}

fn set_property<O, P>(object: &O, parent: Option<&P>, prop: &str, mut value: Value)
where
    O: IsA<Widget> + Cast,
    P: IsA<Container> + Cast,
{
    let object: &Widget = object.upcast_ref();

    // Handle special case hacks
    if let ("image", Some(name)) = (prop, value.get::<String>()) {
        value = Image::new_from_icon_name(name.as_str(), IconSize::Button.into()).to_value();
    }

    // Attempt to set a property on the current object
    if let Err(e) = object.set_property(prop, &value) {
        // If that fails, try it as a child property of the parent, if we have one
        if let Some(parent) = parent {
            let parent: &Container = parent.upcast_ref();
            match ffi::set_child_property(parent, &object, prop, &value) {
                Ok(_) => (),
                Err(e) => {
                    // Handle custom child properties
                    if prop == "center"
                        && value.to_value().get::<bool>() == Some(true)
                        && parent.is::<gtk::Box>()
                    {
                        let parent = parent.downcast_ref::<gtk::Box>().unwrap();
                        parent.remove(object);
                        parent.set_center_widget(object);
                    } else {
                        panic!(
                            "invalid property {:?} for {}: {:?}",
                            prop,
                            object.get_type(),
                            e
                        );
                    }
                }
            }
        } else {
            panic!(
                "invalid property {:?} for {}: {:?}",
                prop,
                object.get_type(),
                e
            )
        }
    }
}

impl<Model: 'static + Component + View<Model>> GtkState<Model> {
    pub fn object<T: IsA<Widget>>(&self) -> Option<&T> {
        self.object.downcast_ref()
    }

    pub fn build(vobj: &VObject<Model>, parent: Option<&Container>, scope: &Scope<Model>) -> Self {
        let id = vobj
            .properties
            .get("id")
            .map(|v| v.get::<String>().expect("id property is not a string"));

        // Build this object
        let object = build_obj::<Widget>(vobj.type_, id.as_ref().map(String::as_str));
        // println!("Built {}", object.get_type());

        // Add to parent
        if let Some(parent) = parent {
            parent.add(&object);
        }

        // Apply properties
        for (prop, value) in &vobj.properties {
            if prop != "id" {
                set_property(&object, parent, prop, value.to_owned());
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
            if let Some(window) = object.downcast_ref::<Window>() {
                match vobj.children.len() {
                    2 => {
                        let header =
                            State::build(&vobj.children[0], Some(window.upcast_ref()), scope);
                        window.remove(header.object());
                        window.set_titlebar(header.object());
                        state.children.push(header);
                        let body =
                            State::build(&vobj.children[1], Some(window.upcast_ref()), scope);
                        state.children.push(body);
                    }
                    1 => {
                        let body =
                            State::build(&vobj.children[0], Some(window.upcast_ref()), scope);
                        state.children.push(body);
                    }
                    length => {
                        panic!(
                            "Window must have either one or two children, but found {}",
                            length
                        );
                    }
                }
            } else if let Some(parent) = object.downcast_ref::<Container>() {
                for child_spec in &vobj.children {
                    let child = State::build(child_spec, Some(parent), scope);
                    state.children.push(child);
                }
            } else {
                panic!("non-Container cannot have children: {}", vobj.type_);
            }
        }

        // Show this object
        state.object.show();

        state
    }

    pub fn patch_object(
        &mut self,
        vobj: &VObject<Model>,
        parent: Option<&Container>,
        scope: &Scope<Model>,
    ) {
        // Patch children
        if let Some(parent) = self.object.downcast_ref::<Container>() {
            let mut to_remove = None;
            let mut to_append = Vec::new();
            let mut reconstruct_from = None;
            for index in 0..(self.children.len().max(vobj.children.len())) {
                match (self.children.get_mut(index), vobj.children.get(index)) {
                    (Some(target), Some(spec_item)) => {
                        match **spec_item {
                            VItem::Object(ref spec) => {
                                if target.object().get_type() == spec.type_ {
                                    // Objects have same type; patch down
                                    target.patch(spec_item, Some(&parent), scope);
                                } else {
                                    // Objects are different, need to reconstruct everything from here
                                    reconstruct_from = Some(index);
                                    break;
                                }
                            }
                            VItem::Component(_) => unimplemented!(),
                        }
                    }
                    (Some(_), None) => {
                        // Extraneous Gtk object; delete
                        if to_remove.is_none() {
                            to_remove = Some(index);
                        }
                    }
                    (None, Some(spec)) => {
                        // New spec; construct
                        let state = State::build(spec, Some(&parent), scope);
                        // Special case: the first child of a window with two
                        // children must be added with set_titlebar
                        if let (Some(window), 0, 2) =
                            (parent.downcast_ref::<Window>(), index, vobj.children.len())
                        {
                            window.remove(state.object());
                            window.set_titlebar(state.object());
                        }
                        to_append.push(state);
                    }
                    (None, None) => break,
                }
            }
            if let Some(mut index) = reconstruct_from {
                // Remove all previous children from here onwards
                if self.object.is::<Window>() && index == 0 && self.children.len() == 2 {
                    panic!("Can't remove a title bar widget from an existing Window!");
                }
                for child in self.children.drain(index..) {
                    parent.remove(child.object());
                }
                // Rebuild children from new specs
                for child_spec in vobj.children.iter().skip(index) {
                    let state = State::build(child_spec, Some(&parent), scope);
                    state.object().show();
                    self.children.push(state);
                }
            } else {
                // Remove children flagged as extraneous
                if let Some(remove_from) = to_remove {
                    if self.object.is::<Window>() && remove_from == 1 && self.children.len() == 2 {
                        panic!("Can't remove a title bar widget from an existing Window!");
                    }
                    for child in self.children.drain(remove_from..) {
                        parent.remove(child.object());
                    }
                }
                // Or append newly constructed children
                if self.object.is::<Window>() && !to_append.is_empty() && self.children.len() == 1 {
                    panic!("Can't add a title bar widget to an existing Window!");
                }
                for child in to_append {
                    child.object().show();
                    self.children.push(child);
                }
            }
        }

        // Patch properties
        self.patch_properties(&vobj.properties, parent);

        // Patch handlers
        self.patch_handlers(&vobj.handlers, scope)
    }

    fn patch_properties(&mut self, properties: &OrdMap<String, Value>, parent: Option<&Container>) {
        for (prop, value) in properties {
            if prop != "id" {
                match self.props.get(prop) {
                    Some(old_value) => {
                        if !eq_values(old_value, value) {
                            set_property(&self.object, parent, prop, value.clone());
                        }
                    }
                    None => set_property(&self.object, parent, prop, value.clone()),
                }
            }
        }
        self.props = properties.clone();
    }

    fn patch_handlers(
        &mut self,
        handlers: &OrdMap<String, OrdSet<Rc<SignalHandler<Model>>>>,
        scope: &Scope<Model>,
    ) {
        for signal_action in handlers.diff(&self.handlers) {
            match signal_action {
                DiffItem::Add((signal, handlers)) => {
                    for handler in handlers {
                        handler.connect(signal.as_str(), self.object.upcast_ref(), scope.clone());
                    }
                }
                DiffItem::Remove((_, handlers)) => {
                    for handler in handlers {
                        handler.disconnect(self.object.upcast_ref());
                    }
                }
                DiffItem::Update {
                    old: (old_signal, old_handlers),
                    new: (signal, handlers),
                } => {
                    debug_assert_eq!(old_signal, signal);
                    for action in handlers.diff(old_handlers) {
                        match action {
                            DiffItem::Add(handler) => handler.connect(
                                signal.as_str(),
                                self.object.upcast_ref(),
                                scope.clone(),
                            ),
                            DiffItem::Remove(handler) => {
                                handler.disconnect(self.object.upcast_ref())
                            }
                            DiffItem::Update {
                                old: old_handler,
                                new: new_handler,
                            } => {
                                old_handler.disconnect(self.object.upcast_ref());
                                new_handler.connect(
                                    signal.as_str(),
                                    self.object.upcast_ref(),
                                    scope.clone(),
                                )
                            }
                        }
                    }
                }
            }
        }
        self.handlers = handlers.clone();
    }
}
