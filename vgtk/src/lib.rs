mod callback;
mod component;
pub mod ext;
mod mainloop;
mod menu_builder;
pub mod properties;
mod scope;
mod vdom;
mod vnode;

use proc_macro_hack::proc_macro_hack;
#[proc_macro_hack(support_nested)]
pub use vgtk_macros::gtk;

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

use crate::component::{ComponentMessage, ComponentTask, PartialComponentTask};
use crate::mainloop::{GtkMainLoop, MainLoop};

pub use crate::callback::Callback;
pub use crate::component::{current_object, Component};
pub use crate::menu_builder::{menu, MenuBuilder};
pub use crate::scope::Scope;
pub use crate::vnode::{PropTransform, VComponent, VHandler, VNode, VObject, VProperty};

thread_local! {
    pub static MAIN_LOOP: GtkMainLoop = GtkMainLoop::new(MainContext::default());
}

/// Signal the main loop to terminate with the given return code.
pub fn main_quit(return_code: i32) {
    MAIN_LOOP.with(|main_loop| main_loop.quit(return_code))
}

/// Run an `Application` component until termination.
pub fn run<C: 'static + Component>() -> i32 {
    gtk::init().expect("GTK failed to initialise");
    let partial_task = PartialComponentTask::<C, ()>::new(Default::default(), None, None);
    let app: Application = partial_task.object().downcast().unwrap_or_else(|_| {
        panic!(
            "The top level object must be an Application, but {} was found.",
            partial_task.object().get_type()
        )
    });
    app.set_default();
    app.register(None as Option<&Cancellable>)
        .expect("unable to register Application");

    let constructor = once(move |_| {
        let (channel, task) = partial_task.finalise();
        MainContext::ref_thread_default().spawn_local(task);
        channel.unbounded_send(ComponentMessage::Mounted).unwrap();
    });

    app.connect_activate(move |_| {
        debug!("Application has activated.");
        constructor(());
    });
    app.activate();
    run_main_loop()
}

/// Launch a modal `Dialog`. The parent window will be blocked until it
/// resolves.
pub fn run_dialog<C: 'static + Component>(
    parent: Option<&GdkWindow>,
) -> impl Future<Output = Result<ResponseType, Canceled>> {
    let (channel, task) = ComponentTask::<C, ()>::new(Default::default(), None, None);
    let dialog: Dialog = task
        .object()
        .downcast()
        .expect("Dialog must be a gtk::Dialog");
    if let Some(parent) = parent {
        dialog.set_parent_window(parent);
    }
    MainContext::ref_thread_default().spawn_local(task);
    let (notify, result) = oneshot::channel();
    dialog.connect_map(move |_| channel.unbounded_send(ComponentMessage::Mounted).unwrap());
    let inner_dialog = dialog.clone();
    let mount = once(move |response| if notify.send(response).is_err() {});
    dialog.connect_response(move |_, response| {
        mount(response);
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

/// Turn an `FnOnce(A)` into an `Fn(A)` that will panic if you call it twice.
fn once<A, F: FnOnce(A)>(f: F) -> impl Fn(A) {
    use std::cell::Cell;
    use std::rc::Rc;

    let f = Rc::new(Cell::new(Some(f)));
    move |value| {
        if let Some(f) = f.take() {
            f(value);
        } else {
            panic!("vgtk::once() function called twice 😒");
        }
    }
}
