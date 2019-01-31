use glib::futures::channel::mpsc::UnboundedSender;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::component::Component;

pub struct Scope<C: Component> {
    muted: Arc<AtomicUsize>,
    channel: UnboundedSender<C::Message>,
}

impl<C: Component> Scope<C> {
    pub(crate) fn new(channel: UnboundedSender<C::Message>) -> Self {
        Scope {
            muted: Default::default(),
            channel,
        }
    }
}

impl<C: Component> Clone for Scope<C> {
    fn clone(&self) -> Self {
        Scope {
            muted: self.muted.clone(),
            channel: self.channel.clone(),
        }
    }
}

impl<C: Component> Scope<C> {
    pub(crate) fn inherit<Child: Component>(
        &self,
        channel: UnboundedSender<Child::Message>,
    ) -> Scope<Child> {
        Scope {
            muted: self.muted.clone(),
            channel,
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::SeqCst) > 0
    }

    pub fn mute(&self) {
        self.muted.fetch_add(1, Ordering::SeqCst);
    }

    pub fn unmute(&self) {
        self.muted.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn send_message(&self, msg: C::Message) {
        println!("Scope::send_message {:?} {:?}", self.is_muted(), msg);
        if !self.is_muted() {
            self.channel
                .unbounded_send(msg)
                .expect("unable to send message to unbounded channel!")
        }
    }
}
