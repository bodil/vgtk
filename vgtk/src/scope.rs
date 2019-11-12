use std::any::TypeId;
use std::sync::{
    atomic::{AtomicPtr, AtomicUsize, Ordering},
    Arc,
};

use colored::Colorize;
use log::debug;

use futures::channel::mpsc::UnboundedSender;

use crate::component::{Component, ComponentTask};

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

    pub fn send_message(&self, msg: C::Message) {
        debug!(
            "{} {}: {}",
            format!(
                "Scope::send_message{}",
                if self.is_muted() { " [muted]" } else { "" }
            )
            .green(),
            self.name.magenta().bold(),
            format!("{:?}", msg).bright_white().bold()
        );
        if !self.is_muted() {
            self.channel
                .unbounded_send(msg)
                .expect("unable to send message to unbounded channel!")
        }
    }

    pub fn name(&self) -> &str {
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
                #[allow(clippy::cast_ptr_alignment)]
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
    // pub fn try_into<C: 'static + Component>(self) -> Result<Box<Scope<C>>, Self> {
    //     if TypeId::of::<C::Properties>() == self.type_id {
    //         let ptr = self.ptr.swap(std::ptr::null_mut(), Ordering::SeqCst);
    //         if ptr.is_null() {
    //             panic!("AnyScope: can't consume dropped value")
    //         } else {
    //             #[allow(clippy::cast_ptr_alignment)]
    //             Ok(unsafe { Box::from_raw(ptr as *mut Scope<C>) })
    //         }
    //     } else {
    //         Err(self)
    //     }
    // }

    pub fn try_get<C: 'static + Component>(&self) -> Option<&'static Scope<C>> {
        if TypeId::of::<C::Properties>() == self.type_id {
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                (self.ptr.load(Ordering::SeqCst) as *const Scope<C>).as_ref()
            }
        } else {
            None
        }
    }
}
