use std::any::TypeId;
use std::fmt::{Debug, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{
    atomic::{AtomicPtr, AtomicUsize, Ordering},
    Arc,
};

use colored::Colorize;
use log::debug;

use futures::channel::mpsc::{TrySendError, UnboundedSender};

use crate::component::{Component, ComponentTask};

/// A channel for sending messages to a [`Component`][Component].
///
/// [Component]: trait.Component.html
pub struct Scope<C: Component> {
    name: &'static str,
    muted: Arc<AtomicUsize>,
    channel: UnboundedSender<C::Message>,
}

impl<C: Component> Scope<C> {
    pub(crate) fn new(name: &'static str, channel: UnboundedSender<C::Message>) -> Self {
        Scope {
            name,
            muted: Default::default(),
            channel,
        }
    }
}

impl<C: Component> Clone for Scope<C> {
    fn clone(&self) -> Self {
        Scope {
            name: self.name,
            muted: self.muted.clone(),
            channel: self.channel.clone(),
        }
    }
}

impl<C: Component> Eq for Scope<C> {}
impl<C: Component> PartialEq for Scope<C> {
    /// Test whether two `Scope`s are equal.
    ///
    /// Two scopes are considered equal if they belong to the same
    /// component instance, in other words if they send their messages
    /// to the same destination.
    fn eq(&self, other: &Self) -> bool {
        self.channel.same_receiver(&other.channel)
    }
}

impl<C: Component> Hash for Scope<C> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.channel.hash_receiver(h)
    }
}

impl<C: Component> Debug for Scope<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Scope[{}]:({:?})", self.name, self.channel)
    }
}

impl<C: 'static + Component> Scope<C> {
    pub(crate) fn inherit<Child: Component>(
        &self,
        name: &'static str,
        channel: UnboundedSender<Child::Message>,
    ) -> Scope<Child> {
        Scope {
            name,
            muted: self.muted.clone(),
            channel,
        }
    }

    pub(crate) fn is_muted(&self) -> bool {
        self.muted.load(Ordering::SeqCst) > 0
    }

    pub(crate) fn mute(&self) {
        self.muted.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn unmute(&self) {
        self.muted.fetch_sub(1, Ordering::SeqCst);
    }

    pub(crate) fn current_parent() -> Self {
        ComponentTask::<_, C>::current_parent_scope()
    }

    #[inline(always)]
    fn log(&self, message: &C::Message) {
        debug!(
            "{} {}: {}",
            format!(
                "Scope::send_message{}",
                if self.is_muted() { " [muted]" } else { "" }
            )
            .green(),
            self.name.magenta().bold(),
            format!("{:?}", message).bright_white().bold()
        );
    }

    #[doc(hidden)]
    pub fn send_message(&self, message: C::Message) {
        self.log(&message);
        if !self.is_muted() {
            self.channel
                .unbounded_send(message)
                .expect("channel has gone unexpectedly out of scope!");
        }
    }

    /// Attempt to send a message to the component this `Scope` belongs to.
    ///
    /// This should always succeed if the component is running.
    ///
    /// If you receive an error, this generally means the component has
    /// unmounted and the scope has become invalid.
    ///
    /// If the message is sent successfully, it will show up at your
    /// component's [`Component::update()`][update] method presently.
    ///
    /// Never call this from inside a signal handler. It's important that you
    /// follow the usual pattern of returning messages from signal handler
    /// closures, or you risk unexpected side effects and potential infinite
    /// loops.
    ///
    /// [update]: ../trait.Component.html#method.update
    pub fn try_send(&self, message: C::Message) -> Result<(), TrySendError<C::Message>> {
        self.log(&message);
        self.channel.unbounded_send(message)
    }

    /// Get the name of the component this `Scope` belongs to.
    pub fn name(&self) -> &'static str {
        &self.name
    }
}

pub(crate) struct AnyScope {
    type_id: TypeId,
    ptr: AtomicPtr<()>,
    drop: Box<dyn Fn(&mut AtomicPtr<()>) + Send>,
}

impl<C: 'static + Component> From<Scope<C>> for AnyScope {
    fn from(scope: Scope<C>) -> Self {
        let ptr = AtomicPtr::new(Box::into_raw(Box::new(scope)) as *mut ());
        let drop = |ptr: &mut AtomicPtr<()>| {
            let ptr = ptr.swap(std::ptr::null_mut(), Ordering::SeqCst);
            if !ptr.is_null() {
                #[allow(unsafe_code)]
                let scope = unsafe { Box::from_raw(ptr as *mut Scope<C>) };
                std::mem::drop(scope)
            }
        };
        AnyScope {
            type_id: TypeId::of::<C::Properties>(),
            ptr,
            drop: Box::new(drop),
        }
    }
}

impl Drop for AnyScope {
    fn drop(&mut self) {
        (self.drop)(&mut self.ptr)
    }
}

impl AnyScope {
    pub(crate) fn try_get<C: 'static + Component>(&self) -> Option<&'static Scope<C>> {
        if TypeId::of::<C::Properties>() == self.type_id {
            #[allow(unsafe_code)]
            unsafe {
                (self.ptr.load(Ordering::Relaxed) as *const Scope<C>).as_ref()
            }
        } else {
            None
        }
    }
}
