use glib::prelude::*;
use glib::{Object, SignalHandlerId, Value};
use std::cell::Cell;
use std::cmp::Ordering;

use std::sync::Arc;

use component::{Component, Scope};

pub struct Event {
    pub source: Object,
    pub args: Vec<Value>,
}

pub struct SignalHandler<C: Component> {
    id: String,
    handler_id: Cell<Option<SignalHandlerId>>,
    handler: Arc<Fn(Event) -> C::Message>,
}

impl<C: Component> PartialEq for SignalHandler<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<C: Component> Eq for SignalHandler<C> {}

impl<C: Component> PartialOrd for SignalHandler<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Component> Ord for SignalHandler<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<C: Component + 'static> SignalHandler<C> {
    pub fn new<F, S>(id: S, f: F) -> Self
    where
        F: Fn(Event) -> C::Message + 'static,
        S: Into<String>,
    {
        let id = id.into();
        SignalHandler {
            id: id,
            handler_id: Cell::new(None),
            handler: Arc::new(f),
        }
    }

    pub fn connect<'a, S: Into<&'a str>>(&self, signal: S, obj: &Object, scope: Scope<C>) {
        let signal = signal.into();
        let handler = self.handler.clone();

        let f: Arc<Fn(Event) -> C::Message + Send + Sync + 'static> =
            unsafe { Arc::from_raw(Arc::into_raw(handler) as *mut _) };

        self.handler_id.set(Some(
            obj.connect(signal, false, move |args: &[Value]| {
                let source: Object = args[0].get().expect("event args[0] was not an Object");
                let event = Event {
                    source,
                    args: args[1..].to_owned(),
                };
                let msg = f(event);
                scope.send_message(msg);
                None
            })
            .unwrap_or_else(|_e| panic!("invalid signal {:?} on {}", signal, obj.get_type())),
        ));
    }

    pub fn disconnect(&self, obj: &Object) {
        if let Some(id) = self.handler_id.replace(None) {
            obj.disconnect(id);
        }
    }
}
