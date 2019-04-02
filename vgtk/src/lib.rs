#![feature(async_closure)]

mod callback;
mod component;
mod event;
mod ffi;
mod mainloop;
mod scope;
pub mod vcomp;
mod vdom;
pub mod vnode;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use vgtk_macros::gtk;

use std::cell::Cell;
use std::sync::Mutex;

use gio::prelude::*;
use gio::{ApplicationFlags, Cancellable};
use glib::futures::{
    channel::oneshot::{self, Canceled},
    Future,
};
use glib::prelude::*;
use glib::MainContext;
use gtk::prelude::*;
use gtk::{Application, Window};

use crate::component::{ComponentMessage, ComponentTask};

pub use crate::callback::Callback;
pub use crate::component::Component;
pub use crate::event::{Event, SignalHandler};
pub use crate::mainloop::{GtkMainLoop, MainLoop};
pub use crate::scope::Scope;
pub use crate::vcomp::VComponent;

thread_local! {
    pub static MAIN_LOOP: GtkMainLoop = GtkMainLoop::new(MainContext::default());
}

/// Signal the main loop to terminate with the given return code.
pub fn main_quit(return_code: i32) {
    MAIN_LOOP.with(|main_loop| main_loop.quit(return_code))
}

/// Launch a `Window` component on a fresh application and run its main loop
/// until termination.
///
/// This is a shortcut for `Application::new()` (and etc), `vgtk::open` and
/// `vgtk::run`, if you're building a simple app that doesn't need much
/// interaction with the application or multiple toplevel windows.
pub fn go<C: 'static + Component>(name: &str, flags: ApplicationFlags) -> i32 {
    let app = Application::new(Some(name), flags).expect("Unable to create GtkApplication");
    app.set_default();
    app.register(None as Option<&Cancellable>)
        .expect("application already running");
    open::<C>(&app);
    app.activate();
    run()
}

/// Launch a `Window` component and open it when the `Application` activates.
///
/// You can call this at any time, regardless of whether the application has
/// activated, but you must call it at least once before trying to activate the
/// app.
pub fn open<C: 'static + Component>(
    app: &Application,
) -> impl Future<Output = Result<Window, Canceled>> {
    let (notify, result) = oneshot::channel();
    // Cell song and dance here because the signal handler can be called
    // repeatedly. It shouldn't be called concurrently from different threads,
    // at least not by Gtk, but just in case, let's Mutex up.
    let notify = Mutex::new(Cell::new(Some(notify)));
    app.connect_activate(move |_| {
        let (_scope, channel, task) = ComponentTask::<C, C>::new(Default::default(), None, None);
        let window: Window = task
            .widget()
            .downcast()
            .expect("Application top level widget must be a Window");
        MAIN_LOOP.with(|main_loop| main_loop.spawn(task));
        window.show_all();
        channel.unbounded_send(ComponentMessage::Mounted).unwrap();
        if let Some(notify) = notify.lock().unwrap().take() {
            // The caller might not have cared about the return value, so the
            // receiver might be gone when we try to send to it. In this case,
            // like the user, we don't care if it fails.
            if notify.send(window).is_err() {
                // We don't care.
            }
        }
    });
    result
}

/// Run the Gtk main loop until termination.
pub fn run() -> i32 {
    MAIN_LOOP.with(mainloop::MainLoop::run)
}

// #[macro_export]
// macro_rules! gtk {
//     ( $stack:ident (< $class:ty : $($tail:tt)*)) => {
//         let mut vcomp = $crate::VComponent::new::<$class>();
//         let mut props = <$class as Component>::Properties::default();
//         gtk!{ @component vcomp props $stack ($class) ($($tail)*) }
//     };
//     (@component $vcomp:ident $props:ident $stack:ident ($class:ty) ( $prop:ident = $value:expr, $($tail:tt)* )) => {
//         $props.$prop = $crate::vcomp::PropTransform::transform(&$vcomp, $value);
//         gtk!{ @component $vcomp $props $stack ($class) ($($tail)*) }
//     };
//     (@component $vcomp:ident $props:ident $stack:ident ($class:ty) (/ > $($tail:tt)*)) => {
//         $vcomp.set_props::<$class>($props);
//         if !$stack.is_empty() {
//             match $stack.last_mut().unwrap() {
//                 $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Component($vcomp)),
//                 $crate::VItem::Component(_) => panic!("Components can't have children"),
//             }
//         } else {
//             panic!("Component can't be a top level item");
//         }
//         gtk!{ $stack ($($tail)*) }
//     };
//     ( $stack:ident (< $class:ident $($tail:tt)*)) => {
//         let mut obj = $crate::VObject::new($class::static_type());
//         gtk!{ @obj obj $class $stack ($($tail)*) }
//     };
//     (@obj $obj:ident $class:ident $stack:ident ( on $signal:ident = |$args:pat| $handler:expr, $($tail:tt)* )) => {
//         let id = format!("{}:{}:{}:{}", file!(), module_path!(), line!(), column!());
//         let handler = $crate::SignalHandler::new(id, move |$args| $handler);
//         $obj.add_handler(stringify!($signal), handler);
//         gtk!{ @obj $obj $class $stack ($($tail)*) }
//     };
//     (@obj $obj:ident $class:ident $stack:ident ( on $signal:ident = $handler:expr, $($tail:tt)* )) => {
//         let id = format!("{}:{}:{}:{}", file!(), module_path!(), line!(), column!());
//         let handler = $crate::SignalHandler::new(id, $handler);
//         $obj.add_handler(stringify!($signal), handler);
//         gtk!{ @obj $obj $class $stack ($($tail)*) }
//     };
//     (@obj $obj:ident $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {
//         $obj.set_property(stringify!($prop), &$value);
//         gtk!{ @obj $obj $class $stack ($($tail)*) }
//     };
//     (@obj $obj:ident $class:ident $stack:ident (/ > $($tail:tt)*)) => {
//         if !$stack.is_empty() {
//             match $stack.last_mut().unwrap() {
//                 $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Object($obj)),
//                 $crate::VItem::Component(_) => panic!("Components can't have children"),
//             }
//         } else {
//             $stack.push(VItem::Object($obj));
//         }
//         gtk!{ $stack ($($tail)*) }
//     };
//     (@obj $obj:ident $class:ident $stack:ident (> $($tail:tt)*)) => {
//         $stack.push(VItem::Object($obj));
//         gtk!{ $stack ($($tail)*) }
//     };
//     ( $stack:ident (< / $class:ident > $($tail:tt)*)) => {
//         match $stack.pop().unwrap() {
//             $crate::VItem::Object(child) => {
//                 debug_assert_eq!(child.type_, $class::static_type(), "you forgot to close a tag, closed one twice, or used `<tag/>` for a parent");
//                 if !$stack.is_empty() {
//                     match $stack.last_mut().unwrap() {
//                         $crate::VItem::Object(parent) => parent.add_child($crate::VItem::Object(child)),
//                         $crate::VItem::Component(_) => panic!("Components can't have children"),
//                     }
//                 } else {
//                     $stack.push(VItem::Object(child));
//                 }
//             }
//             $crate::VItem::Component(_) => panic!("Components can't have children"),
//         }
//         gtk!{ $stack ($($tail)*) }
//     };
//     ( $stack:ident ({ for $eval:expr } $($tail:tt)*)) => {
//         {
//             let mut nodes = $eval;
//             if !$stack.is_empty() {
//                 match $stack.last_mut().unwrap() {
//                     $crate::VItem::Object(parent) => {
//                         for child in nodes {
//                             parent.add_child(child);
//                         }
//                     }
//                     $crate::VItem::Component(_) => panic!("Components can't have children"),
//                 }
//             } else {
//                 if let Some(node) = nodes.next() {
//                     debug_assert!(nodes.next().is_none(), "only one top level widget is allowed");
//                     $stack.push(node);
//                 } else {
//                     panic!("for expression in gtk! macro produced no child nodes");
//                 }
//             }
//         }
//         gtk!{ $stack ($($tail)*) }
//     };
//     ( $stack:ident ({ $($rule:expr => $body:expr)* } $($tail:tt)*)) => {
//         {
//             $(
//                 if $rule {
//                     let mut node = $body;
//                     if !$stack.is_empty() {
//                         match $stack.last_mut().unwrap() {
//                             $crate::VItem::Object(parent) => parent.add_child(node),
//                             $crate::VItem::Component(_) => panic!("Components can't have children"),
//                         }
//                     } else {
//                         $stack.push(node);
//                     }
//                 }
//             )*
//         }
//         gtk!{ $stack ($($tail)*) }
//     };
//     ($stack:ident ()) => {
//         $stack.pop().expect("empty gtk! macro")
//     };
//     ($($tail:tt)*) => {{
//         let mut stack: Vec<$crate::VItem<_>> = Vec::new();
//         gtk!{ stack ($($tail)*) }
//     }};
// }
