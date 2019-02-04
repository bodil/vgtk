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
mod mainloop;
mod scope;
pub mod vcomp;
mod vdom;
mod vitem;
mod vobject;

use gio::prelude::*;
use gio::ApplicationFlags;
use glib::prelude::*;
use glib::MainContext;
use gtk::prelude::*;
use gtk::Window;

use crate::component::{ComponentMessage, ComponentTask};

pub use crate::callback::Callback;
pub use crate::component::Component;
pub use crate::event::{Event, SignalHandler};
pub use crate::mainloop::{GtkMainLoop, MainLoop};
pub use crate::scope::Scope;
pub use crate::vcomp::VComponent;
pub use crate::vitem::VItem;
pub use crate::vobject::VObject;

thread_local! {
    pub static MAIN_LOOP: GtkMainLoop = GtkMainLoop::new(MainContext::default());
}

pub fn main_quit(return_code: i32) {
    MAIN_LOOP.with(|main_loop| main_loop.quit(return_code))
}

pub fn run<C: 'static + Component>(name: &str, flags: ApplicationFlags, _args: &[String]) -> i32 {
    MAIN_LOOP.with(|main_loop| {
        let app = gtk::Application::new(name, flags).expect("Unable to create GtkApplication");
        let (_scope, channel, task) = ComponentTask::<C, C>::new(Default::default(), None, None);
        let window: Window = task
            .widget()
            .downcast()
            .expect("Application's top level widget must be a Window");
        main_loop.spawn(task);
        app.connect_activate(move |_| {
            window.show_all();
            channel.unbounded_send(ComponentMessage::Mounted).unwrap();
        });
        app.set_default();
        app.register(None).expect("application already running");
        app.activate();
        main_loop.run()
    })
}

#[macro_export]
macro_rules! gtk {
    ( $stack:ident (< $class:ident : $($tail:tt)*)) => {
        let mut vcomp = $crate::VComponent::new::<$class>();
        let mut props = <$class as Component>::Properties::default();
        gtk!{ @component vcomp props $class $stack ($($tail)*) }
    };
    (@component $vcomp:ident $props:ident $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {
        $props.$prop = $crate::vcomp::PropTransform::transform(&$vcomp, $value);
        gtk!{ @component $vcomp $props $class $stack ($($tail)*) }
    };
    (@component $vcomp:ident $props:ident $class:ident $stack:ident (/ > $($tail:tt)*)) => {
        $vcomp.set_props::<$class>($props);
        if !$stack.is_empty() {
            match $stack.last_mut().unwrap() {
                $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Component($vcomp)),
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
