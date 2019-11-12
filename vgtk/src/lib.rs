mod callback;
mod component;
pub mod ext;
mod menu_builder;
#[doc(hidden)]
pub mod properties;
#[doc(hidden)]
pub mod scope;
mod vdom;
#[doc(hidden)]
pub mod vnode;

use proc_macro_hack::proc_macro_hack;

/// Generate a virtual component tree.
#[proc_macro_hack(support_nested)]
pub use vgtk_macros::gtk;

use gio::prelude::*;
use gio::Cancellable;
use glib::MainContext;
use gtk::prelude::*;
use gtk::{
    Application, ButtonsType, Dialog, DialogFlags, MessageDialog, MessageType, ResponseType, Window,
};

use futures::channel::oneshot::{self, Canceled};
use std::future::Future;

use colored::Colorize;
use log::debug;

use crate::component::{ComponentMessage, ComponentTask, PartialComponentTask};

pub use crate::callback::Callback;
pub use crate::component::{current_object, current_window, Component, UpdateAction};
pub use crate::menu_builder::{menu, MenuBuilder};
pub use crate::vnode::VNode;

/// Re-exports of Gtk and its associated libraries.
///
/// It is recommended that you use these rather than pulling them in as
/// dependencies of your own project, to avoid versioning conflicts.
pub mod lib {
    pub use gdk;
    pub use gdk_pixbuf;
    pub use gio;
    pub use glib;
    pub use gtk;
}

/// Run an `Application` component until termination.
///
/// This is generally the function you'll call to get everything up and running.
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

    let const_app = app.clone();

    let constructor = once(move |_| {
        let (channel, task) = partial_task.finalise();
        MainContext::ref_thread_default().spawn_local(task);
        channel.unbounded_send(ComponentMessage::Mounted).unwrap();
        const_app.connect_shutdown(move |_| {
            channel.unbounded_send(ComponentMessage::Unmounted).unwrap();
        });
    });

    app.connect_activate(move |_| {
        debug!("{}", "Application has activated.".bright_blue());
        constructor(());
    });

    let args: Vec<String> = std::env::args().collect();
    app.run(&args)
}

/// Launch a `Dialog` component as a modal dialog.
///
/// The parent window will be blocked until it resolves.
///
/// It returns a `Future` which resolves either to `Ok(ResponseType)` when the
/// `response` signal is emitted, or to `Err(Canceled)` if the dialog is
/// destroyed before the user responds to it.
pub fn run_dialog<C: 'static + Component>(
    parent: Option<&Window>,
) -> impl Future<Output = Result<ResponseType, Canceled>> {
    let (channel, task) = ComponentTask::<C, ()>::new(Default::default(), None, None);
    let dialog: Dialog = task
        .object()
        .unwrap()
        .downcast()
        .expect("Dialog must be a gtk::Dialog");
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
    }
    MainContext::ref_thread_default().spawn_local(task);
    let (notify, result) = oneshot::channel();
    channel.unbounded_send(ComponentMessage::Mounted).unwrap();
    let resolve = once(move |response| if notify.send(response).is_err() {});
    dialog.connect_response(move |_, response| {
        resolve(response);
        channel.unbounded_send(ComponentMessage::Unmounted).unwrap()
    });
    dialog.present();
    result
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

/// Tell the running `Application` to quit.
///
/// This calls `Application::quit()` on the current default `Application`. It
/// will cause the `vgtk::run()` in charge of that `Application` to terminate.
pub fn quit() {
    gio::Application::get_default()
        .expect("no default Application!")
        .quit();
}

/// Connect a GLib signal to a `Future`.
///
/// This macro takes a GLib object and the name of a method to connect it to a
/// signal (generally of the form `connect_signal_name`), and generates an
/// `async` block that will resolve with the emitted value the first time the
/// signal is emitted.
///
/// The output type of the async block is `Result<T, Canceled>`, where `T` is
/// the type of the emitted value (the second argument to the callback
/// `connect_signal_name` takes). It will produce `Err(Canceled)` if the object
/// is destroyed before the signal is emitted.
#[macro_export]
macro_rules! on_signal {
    ($object:expr, $connect:ident) => {
        async {
            let (notify, result) = futures::channel::oneshot::channel();
            let state = std::sync::Arc::new(std::sync::Mutex::new((None, Some(notify))));
            let state_outer = state.clone();
            let id = $object.$connect(move |obj, value| {
                let mut lock = state.lock().unwrap();
                if let Some(notify) = lock.1.take() {
                    if notify.send(value).is_ok() {}
                }
                if let Some(handler) = lock.0.take() {
                    obj.disconnect(handler);
                }
            });
            state_outer.lock().unwrap().0 = Some(id);
            result.await
        }
    };
}

/// Connect a GLib signal to a `Stream`.
///
/// This macro takes a GLib object and the name of a method to connect it to a
/// signal (generally of the form `connect_signal_name`), and generates a
/// `Stream` that will produce a value every time the signal is emitted.
///
/// The output type of the stream is the type of the emitted value (the second
/// argument to the callback `connect_signal_name` takes). The stream will
/// terminate when the object it's connected to is destroyed.
#[macro_export]
macro_rules! stream_signal {
    ($object:expr, $connect:ident) => {{
        let (input, output) = futures::channel::mpsc::unbounded();
        $object.$connect(move |_, value| if input.unbounded_send(value).is_ok() {});
        output
    }};
}

/// Open a simple `MessageDialog`.
///
/// The arguments are passed directly to `MessageDialog::new()`. The `is_markup`
/// flag, if set, will interpret the `message` as markup rather than plain text
/// (see `MessageDialog::set_markup()`).
///
/// It returns a `Future` which will resolve to the `ResponseType` the user
/// responds with.
pub async fn message_dialog<W, S>(
    parent: Option<&W>,
    flags: DialogFlags,
    message_type: MessageType,
    buttons: ButtonsType,
    is_markup: bool,
    message: S,
) -> ResponseType
where
    W: IsA<Window>,
    S: AsRef<str>,
{
    let dialog = MessageDialog::new(parent, flags, message_type, buttons, message.as_ref());
    dialog.set_modal(true);
    if is_markup {
        dialog.set_markup(message.as_ref());
    }
    dialog.show();
    let response = on_signal!(dialog, connect_response).await;
    dialog.destroy();
    response.unwrap()
}
