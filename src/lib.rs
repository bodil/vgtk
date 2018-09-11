extern crate gio;
extern crate glib;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate gtk;
extern crate gtk_sys as gtk_ffi;

mod component;
mod event;
mod ffi;
mod vobject;

use gio::prelude::*;
use gio::ApplicationFlags;
use glib::prelude::*;
use gtk::prelude::*;
use gtk::{idle_add, Window, WindowType};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub use component::{Component, Scope, View};
pub use event::{Event, SignalHandler};
pub use vobject::VObject;

pub struct Application<C: Component> {
    model: C,
    toplevel: Window,
    queue: Arc<Mutex<VecDeque<C::Message>>>,
}

impl<C: 'static + Component + View<C>> Application<C> {
    pub fn run(name: &str, flags: ApplicationFlags, args: &[String]) -> i32 {
        let app = gtk::Application::new(name, flags).expect("Unable to create GtkApplication");
        let app_init = app.clone();
        app.connect_activate(move |_| {
            let mut state = Application {
                model: C::default(),
                toplevel: Window::new(WindowType::Toplevel),
                queue: Default::default(),
            };
            let scope = Scope {
                queue: state.queue.clone(),
            };
            let view = state.model.view();
            let window: Window = view.build(&scope);
            app_init.add_window(&window);
            window.show_all();
            state.toplevel = window;
            let app_loop = app_init.clone();
            idle_add(move || {
                // TODO this is busy waiting, maybe do better
                if app_loop.get_windows().is_empty() {
                    return Continue(false);
                }
                let mut q = state.queue.lock().unwrap();
                let mut render = false;
                while let Some(msg) = q.pop_front() {
                    if state.model.update(msg) {
                        render = true;
                    }
                }
                if render {
                    state.model.view().patch(&scope, &state.toplevel);
                }
                Continue(true)
            });
        });
        app.activate();
        app.run(args)
    }
}

#[macro_export]
macro_rules! gtk {
    ( $stack:ident (< $class:ident $($tail:tt)*)) => {
        {
            let obj = $crate::VObject::new($class::static_type());
            $stack.push(obj);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident ( on $signal:ident = |$args:pat| $handler:expr, $($tail:tt)* )) => {
        {
            let obj = $stack.last_mut().expect("stack was empty!");
            let handler = $crate::SignalHandler::new(move |$args| $handler);
            obj.add_handler(stringify!($signal), handler);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident ( on $signal:ident = $handler:expr, $($tail:tt)* )) => {
        {
            let obj = $stack.last_mut().expect("stack was empty!");
            let handler = $crate::SignalHandler::new($handler);
            obj.add_handler(stringify!($signal), handler);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {
        {
            let obj = $stack.last_mut().expect("stack was empty!");
            obj.set_property(stringify!($prop), &$value);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident (/ > $($tail:tt)*)) => {
        {
            let child = $stack.pop().unwrap();
            if !$stack.is_empty() {
                let parent = $stack.last_mut().unwrap();
                parent.add_child(child);
            } else {
                $stack.push(child);
            }
        }
        gtk!{ $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident (> $($tail:tt)*)) => {
        gtk!{ $stack ($($tail)*) }
    };
    ( $stack:ident (< / $class:ident > $($tail:tt)*)) => {
        {
            let child = $stack.pop().unwrap();
            debug_assert!(child.type_ == $class::static_type());
            if !$stack.is_empty() {
                let parent = $stack.last_mut().unwrap();
                parent.add_child(child);
            } else {
                $stack.push(child);
            }
        }
        gtk!{ $stack ($($tail)*) }
    };
    ($stack:ident ()) => {
        $stack.pop().expect("empty gtk! macro")
    };
    ($($tail:tt)*) => {{
        let mut stack = Vec::new();
        gtk!{ stack ($($tail)*) }
    }}
}
