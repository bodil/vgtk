mod callback;
mod component;
pub mod ext;
mod menu_builder;
pub mod properties;
mod scope;
mod vdom;
mod vnode;

use proc_macro_hack::proc_macro_hack;
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
pub use crate::scope::Scope;
pub use crate::vnode::{PropTransform, VComponent, VHandler, VNode, VObject, VProperty};

pub mod lib {
    pub use gdk;
    pub use gdk_pixbuf;
    pub use gio;
    pub use glib;
    pub use gtk;
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

/// Launch a modal `Dialog`. The parent window will be blocked until it
/// resolves.
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

pub fn quit() {
    gio::Application::get_default()
        .expect("no default Application!")
        .quit();
}

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
