use glib::prelude::*;
use glib::{Object, SignalHandlerId, Value};

use std::sync::Arc;

use component::{Component, Scope};

pub struct Event {
    pub source: Object,
    pub args: Vec<Value>,
}

pub struct SignalHandler<C: Component> {
    handler: Arc<Fn(Event) -> C::Message>,
}

impl<C: Component + 'static> SignalHandler<C> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Event) -> C::Message + 'static,
    {
        SignalHandler {
            handler: Arc::new(f),
        }
    }

    pub fn connect<'a, S: Into<&'a str>>(
        &self,
        signal: S,
        obj: &Object,
        scope: Scope<C>,
    ) -> SignalHandlerId {
        let signal = signal.into();
        let handler = self.handler.clone();

        let f: Arc<Fn(Event) -> C::Message + Send + Sync + 'static> =
            unsafe { Arc::from_raw(Arc::into_raw(handler) as *mut _) };

        obj.connect(signal, false, move |args: &[Value]| {
            let source: Object = args[0].get().expect("event args[0] was not an Object");
            let event = Event {
                source,
                args: args[1..].to_owned(),
            };
            let msg = f(event);
            scope.send_message(msg);
            None
        }).unwrap_or_else(|_e| panic!("invalid signal {:?} on {}", signal, obj.get_type()))
    }
}
