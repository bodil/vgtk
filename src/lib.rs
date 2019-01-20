extern crate gio;
extern crate glib;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate gtk;
extern crate gtk_sys as gtk_ffi;
extern crate im;

mod callback;
mod component;
mod event;
mod ffi;
pub mod vcomp;
mod vdom;
mod vitem;
mod vobject;

use gio::prelude::*;
use gio::ApplicationFlags;
use glib::prelude::*;
use gtk::prelude::*;
use gtk::Window;

use crate::vdom::State;

pub use crate::callback::Callback;
pub use crate::component::{Component, Scope, View};
pub use crate::event::{Event, SignalHandler};
pub use crate::vcomp::VComponent;
pub use crate::vitem::VItem;
pub use crate::vobject::VObject;

pub struct Application<C: Component + View<C>> {
    model: C,
    ui_state: State<C>,
    scope: Scope<C>,
}

impl<C: 'static + Component + View<C>> Application<C> {
    pub fn run(name: &str, flags: ApplicationFlags, args: &[String]) -> i32 {
        let app = gtk::Application::new(name, flags).expect("Unable to create GtkApplication");
        let app_init = app.clone();
        app.connect_activate(move |_| {
            let scope = Scope::default();
            let model = C::default();
            let initial_view = model.view();
            let ui_state = State::build(&initial_view, None, &scope);
            let mut state = Application {
                model,
                ui_state,
                scope: scope.clone(),
            };
            {
                let window: &Window = state
                    .ui_state
                    .object()
                    .downcast_ref()
                    .expect("Application's top level widget must be a Window");
                app_init.add_window(window);
                window.show_all();
            }
            let app_loop = app_init.clone();
            timeout_add(5, move || {
                if app_loop.get_windows().is_empty() {
                    return Continue(false);
                }
                let mut render = false;
                {
                    let mut q = scope.queue.lock().unwrap();
                    while let Some(msg) = q.pop_front() {
                        if state.model.update(msg) {
                            render = true;
                        }
                    }
                }
                if render {
                    let new_view = state.model.view();
                    scope.mute();
                    if !state.ui_state.patch(&new_view, None, &scope) {
                        panic!("Cannot change type of toplevel window");
                    }
                    scope.unmute();
                }
                Continue(true)
            });
        });
        app.activate();
        app.run(args)
    }

    pub fn process(&mut self) {
        let mut render = false;
        {
            let mut q = self.scope.queue.lock().unwrap();
            while let Some(msg) = q.pop_front() {
                if self.model.update(msg) {
                    render = true;
                }
            }
        }
        if render {
            let new_view = self.model.view();
            self.scope.mute();
            if !self.ui_state.patch(&new_view, None, &self.scope) {
                panic!("Cannot change type of toplevel window");
            }
            self.scope.unmute();
        }
    }
}

#[macro_export]
macro_rules! gtk {
    ( $stack:ident (< $class:ident : $($tail:tt)*)) => {
        let mut props = <$class as Component>::Properties::default();
        let mut activators = Vec::new();
        gtk!{ @component props activators $class $stack ($($tail)*) }
    };
    (@component $props:ident $act:ident $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {
        $props.$prop = $crate::vcomp::PropTransform::transform(&mut $act, $value);
        gtk!{ @component $props $act $class $stack ($($tail)*) }
    };
    (@component $props:ident $act:ident $class:ident $stack:ident (/ > $($tail:tt)*)) => {
        if !$stack.is_empty() {
            match $stack.last_mut().unwrap() {
                $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Component($crate::VComponent::new::<$class>($props, $act))),
                $crate::VItem::Component(_) => panic!("Components can't have children"),
            }
        } else {
            panic!("Component can't be a top level item");
        }
        gtk!{ $stack ($($tail)*) }
    };
    ( $stack:ident (< $class:ident $($tail:tt)*)) => {
        let mut obj = $crate::VObject::new($class::static_type());
        gtk!{ @obj obj $class $stack ($($tail)*) }
    };
    (@obj $obj:ident $class:ident $stack:ident ( on $signal:ident = |$args:pat| $handler:expr, $($tail:tt)* )) => {
        let id = format!("{}:{}:{}:{}", file!(), module_path!(), line!(), column!());
        let handler = $crate::SignalHandler::new(id, move |$args| $handler);
        $obj.add_handler(stringify!($signal), handler);
        gtk!{ @obj $obj $class $stack ($($tail)*) }
    };
    (@obj $obj:ident $class:ident $stack:ident ( on $signal:ident = $handler:expr, $($tail:tt)* )) => {
        let id = format!("{}:{}:{}:{}", file!(), module_path!(), line!(), column!());
        let handler = $crate::SignalHandler::new(id, $handler);
        $obj.add_handler(stringify!($signal), handler);
        gtk!{ @obj $obj $class $stack ($($tail)*) }
    };
    (@obj $obj:ident $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {
        $obj.set_property(stringify!($prop), &$value);
        gtk!{ @obj $obj $class $stack ($($tail)*) }
    };
    (@obj $obj:ident $class:ident $stack:ident (/ > $($tail:tt)*)) => {
        if !$stack.is_empty() {
            match $stack.last_mut().unwrap() {
                $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Object($obj)),
                $crate::VItem::Component(_) => panic!("Components can't have children"),
            }
        } else {
            $stack.push(VItem::Object($obj));
        }
        gtk!{ $stack ($($tail)*) }
    };
    (@obj $obj:ident $class:ident $stack:ident (> $($tail:tt)*)) => {
        $stack.push(VItem::Object($obj));
        gtk!{ $stack ($($tail)*) }
    };
    ( $stack:ident (< / $class:ident > $($tail:tt)*)) => {
        match $stack.pop().unwrap() {
            $crate::VItem::Object(child) => {
                debug_assert_eq!(child.type_, $class::static_type(), "you forgot to close a tag, closed one twice, or used `<tag/>` for a parent");
                if !$stack.is_empty() {
                    match $stack.last_mut().unwrap() {
                        $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Object(child)),
                        $crate::VItem::Component(_) => panic!("Components can't have children"),
                    }
                } else {
                    $stack.push(VItem::Object(child));
                }
            }
            $crate::VItem::Component(_) => panic!("Components can't have children"),
        }
        gtk!{ $stack ($($tail)*) }
    };
    ( $stack:ident ({ for $eval:expr } $($tail:tt)*)) => {
        {
            let mut nodes = $eval;
            if !$stack.is_empty() {
                match $stack.last_mut().unwrap() {
                    $crate::VItem::Object(parent) => {
                        for child in nodes {
                            parent.add_child(child);
                        }
                    }
                    $crate::VItem::Component(_) => panic!("Components can't have children"),
                }
            } else {
                if let Some(node) = nodes.next() {
                    debug_assert!(nodes.next().is_none(), "only one top level widget is allowed");
                    $stack.push(node);
                } else {
                    panic!("for expression in gtk! macro produced no child nodes");
                }
            }
        }
        gtk!{ $stack ($($tail)*) }
    };
    ( $stack:ident ({ $($rule:expr => $body:expr)* } $($tail:tt)*)) => {
        {
            $(
                if $rule {
                    let mut node = $body;
                    if !$stack.is_empty() {
                        match $stack.last_mut().unwrap() {
                            $crate::VItem::Object(parent) => parent.add_child(node),
                            $crate::VItem::Component(_) => panic!("Components can't have children"),
                        }
                    } else {
                        $stack.push(node);
                    }
                }
            )*
        }
        gtk!{ $stack ($($tail)*) }
    };
    ($stack:ident ()) => {
        $stack.pop().expect("empty gtk! macro")
    };
    ($($tail:tt)*) => {{
        let mut stack: Vec<$crate::VItem<_>> = Vec::new();
        gtk!{ stack ($($tail)*) }
    }};
}
