use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use vitem::VItem;

pub trait Component: Default {
    type Message: Send + Debug;
    type Properties: Clone + Default;
    fn update(&mut self, msg: Self::Message) -> bool;
    fn create(_props: Self::Properties) -> Self {
        Self::default()
    }
    fn change(&mut self, _props: Self::Properties) -> bool {
        unimplemented!()
    }
}

pub trait View<Model: Component> {
    fn view(&self) -> VItem<Model>;
}

pub struct Scope<C: Component> {
    muted: Arc<AtomicBool>,
    pub(crate) queue: Arc<Mutex<VecDeque<C::Message>>>,
}

impl<C: Component> Default for Scope<C> {
    fn default() -> Self {
        Scope {
            muted: Arc::new(AtomicBool::new(false)),
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl<C: Component> Clone for Scope<C> {
    fn clone(&self) -> Self {
        Scope {
            muted: self.muted.clone(),
            queue: self.queue.clone(),
        }
    }
}

impl<C: Component> Scope<C> {
    fn is_muted(&self) -> bool {
        self.muted.load(Ordering::SeqCst)
    }

    pub fn mute(&self) {
        self.muted.store(true, Ordering::SeqCst)
    }

    pub fn unmute(&self) {
        self.muted.store(false, Ordering::SeqCst)
    }

    pub fn send_message(&self, msg: C::Message) {
        println!("Scope::send_message {:?} {:?}", self.is_muted(), msg);
        if !self.is_muted() {
            self.queue.lock().unwrap().push_back(msg)
        }
    }
}
