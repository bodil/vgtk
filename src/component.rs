use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use vobject::VObject;

pub trait Component: Default {
    type Message: Send;
    fn update(&mut self, msg: Self::Message) -> bool;
}

pub trait View<Model: Component> {
    fn view(&self) -> VObject<Model>;
}

pub struct Scope<C: Component> {
    pub(crate) queue: Arc<Mutex<VecDeque<C::Message>>>,
}

impl<C: Component> Clone for Scope<C> {
    fn clone(&self) -> Self {
        Scope {
            queue: self.queue.clone(),
        }
    }
}

impl<C: Component> Scope<C> {
    pub fn send_message(&self, msg: C::Message) {
        self.queue.lock().unwrap().push_back(msg)
    }
}
