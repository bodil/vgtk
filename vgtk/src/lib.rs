mod callback;
mod component;
pub mod ext;
mod mainloop;
pub mod properties;
mod scope;
mod vdom;
mod vnode;

use proc_macro_hack::proc_macro_hack;
#[proc_macro_hack(support_nested)]
pub use vgtk_macros::gtk;

use std::cell::Cell;
use std::sync::Mutex;

use gdk::Window as GdkWindow;
use gio::prelude::*;
use gio::Cancellable;
use glib::futures::{
    channel::oneshot::{self, Canceled},
    Future,
};
use glib::prelude::*;
use glib::MainContext;
use gtk::prelude::*;
use gtk::{Application, Dialog, ResponseType};

use log::debug;

use crate::component::{ComponentMessage, ComponentTask};
use crate::mainloop::{GtkMainLoop, MainLoop};

pub use crate::callback::Callback;
pub use crate::component::{current_object, Component};
pub use crate::scope::Scope;
pub use vnode::{PropTransform, VComponent, VHandler, VNode, VObject, VProperty};

thread_local! {
    pub static MAIN_LOOP: GtkMainLoop = GtkMainLoop::new(MainContext::default());
}

/// Signal the main loop to terminate with the given return code.
pub fn main_quit(return_code: i32) {
    MAIN_LOOP.with(|main_loop| main_loop.quit(return_code))
}

/// Run an `Application` until termination.
pub fn run<C: 'static + Component>() -> i32 {
    gtk::init().expect("GTK failed to initialise");
    let (_scope, channel, view, mut task) =
        ComponentTask::<C, ()>::new_defer(Default::default(), None, None);
    let app: Application = task.object().downcast().unwrap_or_else(|_| {
        panic!(
            "The top level object must be an Application, but {} was found.",
            task.object().get_type()
        )
    });
    app.set_default();
    app.register(None as Option<&Cancellable>)
        .expect("application already running");

    let constructor = Mutex::new(Cell::new(Some(move |app: &Application| {
        task.finalise_deferred(view);
        MainContext::ref_thread_default().spawn_local(task);
        for window in app.get_windows() {
            window.show_all();
        }
        channel.unbounded_send(ComponentMessage::Mounted).unwrap();
    })));

    app.connect_activate(move |app| {
        debug!("Application has activated.");
        if let Some(constructor) = constructor.lock().unwrap().take() {
            constructor(app);
        }
    });
    app.activate();
    run_main_loop()
}

/// Launch a modal `Dialog`. The parent window will be blocked until it
/// resolves.
pub fn run_dialog<C: 'static + Component>(
    parent: Option<&GdkWindow>,
) -> impl Future<Output = Result<ResponseType, Canceled>> {
    let (_scope, channel, task) = ComponentTask::<C, ()>::new(Default::default(), None, None);
    let dialog: Dialog = task
        .object()
        .downcast()
        .expect("Dialog must be a gtk::Dialog");
    if let Some(parent) = parent {
        dialog.set_parent_window(parent);
    }
    MainContext::ref_thread_default().spawn_local(task);
    let (notify, result) = oneshot::channel();
    let notify = Mutex::new(Cell::new(Some(notify)));
    dialog.connect_map(move |_| channel.unbounded_send(ComponentMessage::Mounted).unwrap());
    let inner_dialog = dialog.clone();
    dialog.connect_response(move |_, response| {
        if let Some(notify) = notify.lock().unwrap().take() {
            if notify.send(response).is_err() {}
        }
        inner_dialog.destroy();
    });
    dialog.present();
    result
}

/// Run the Gtk main loop until termination.
pub fn run_main_loop() -> i32 {
    MAIN_LOOP.with(mainloop::MainLoop::run)
}

pub fn icon(name: &str, size: gtk::IconSize) -> gtk::Image {
    gtk::Image::new_from_icon_name(Some(name), size)
}
