mod callback;
mod component;
mod event;
pub mod ext;
mod mainloop;
pub mod properties;
mod scope;
pub mod vcomp;
mod vdom;
pub mod vnode;

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack(support_nested)]
pub use vgtk_macros::gtk;

use std::cell::Cell;
use std::sync::Mutex;

use gdk::Window as GdkWindow;
use gio::prelude::*;
use gio::{ApplicationFlags, Cancellable};
use glib::futures::{
    channel::oneshot::{self, Canceled},
    Future,
};
use glib::prelude::*;
use glib::MainContext;
use gtk::prelude::*;
use gtk::{Application, Dialog, ResponseType, Window};

use crate::component::{ComponentMessage, ComponentTask};

pub use crate::callback::Callback;
pub use crate::component::{current_widget, Component};
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
#[allow(unused_must_use)]
pub fn go<C: 'static + Component>(name: &str, flags: ApplicationFlags) -> i32 {
    let app = Application::new(Some(name), flags).expect("Unable to create GtkApplication");
    app.set_default();
    app.register(None as Option<&Cancellable>)
        .expect("application already running");
    open::<C>(&app);
    app.activate();
    run()
}

pub fn build<C: 'static + Component>() -> Window {
    let (_scope, channel, task) = ComponentTask::<C, ()>::new(Default::default(), None, None);
    let window: Window = task
        .widget()
        .downcast()
        .expect("Application top level widget must be a Window");
    // MAIN_LOOP.with(|main_loop| main_loop.spawn(task));
    MainContext::ref_thread_default().spawn_local(task);
    window.show_all();
    channel.unbounded_send(ComponentMessage::Mounted).unwrap();
    window
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
        let window = build::<C>();
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

pub fn run_dialog<C: 'static + Component>(parent: Option<&GdkWindow>) -> ResponseType {
    let (_scope, channel, task) = ComponentTask::<C, ()>::new(Default::default(), None, None);
    let dialog: Dialog = task
        .widget()
        .downcast()
        .expect("Dialog must be a gtk::Dialog");
    if let Some(parent) = parent {
        dialog.set_parent_window(parent);
    }
    let task = Mutex::new(Cell::new(Some(task)));
    dialog.connect_map(move |_| {
        if let Some(task) = task.lock().unwrap().take() {
            MainContext::ref_thread_default().spawn_local(task);
        }
        channel.unbounded_send(ComponentMessage::Mounted).unwrap();
    });
    let response = dialog.run();
    dialog.destroy();
    response
}

/// Run the Gtk main loop until termination.
pub fn run() -> i32 {
    MAIN_LOOP.with(mainloop::MainLoop::run)
}

pub fn icon(name: &str, size: gtk::IconSize) -> gtk::Image {
    gtk::Image::new_from_icon_name(Some(name), size)
}
