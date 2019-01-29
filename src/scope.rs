use glib::futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    task::Context,
    Async, Future, Never, Poll, Stream, StreamExt,
};
use gtk::Container;
use gtk::Widget;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::component::{Component, View};
use crate::vdom::State;

pub(crate) enum ComponentMessage<C: Component> {
    Update(C::Message),
    Props(C::Properties),
    Mounted,
    Unmounted,
}

impl<C: Component> Clone for ComponentMessage<C> {
    fn clone(&self) -> Self {
        match self {
            ComponentMessage::Update(msg) => ComponentMessage::Update(msg.clone()),
            ComponentMessage::Props(props) => ComponentMessage::Props(props.clone()),
            ComponentMessage::Mounted => ComponentMessage::Mounted,
            ComponentMessage::Unmounted => ComponentMessage::Unmounted,
        }
    }
}

pub struct Scope<C: Component> {
    muted: Arc<AtomicUsize>,
    channel: UnboundedSender<C::Message>,
}

impl<C: Component> Scope<C> {
    fn new(channel: UnboundedSender<C::Message>) -> Self {
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

pub struct ComponentTask<C, P>
where
    C: Component + View<C>,
    P: Component + View<P>,
{
    scope: Scope<C>,
    parent_scope: Option<Scope<P>>,
    state: C,
    ui_state: State<C>,
    channel: Box<Stream<Item = ComponentMessage<C>, Error = Never>>,
}

impl<C, P> ComponentTask<C, P>
where
    C: 'static + Component + View<C>,
    P: Component + View<P>,
{
    pub(crate) fn new(
        props: C::Properties,
        parent: Option<&Container>,
        parent_scope: Option<&Scope<P>>,
    ) -> (Scope<C>, UnboundedSender<ComponentMessage<C>>, Self) {
        let (sys_send, sys_recv) = unbounded();
        let (user_send, user_recv) = unbounded();

        // As `C::Message` must be `Send` but `C::Properties` can't be,
        // we keep two senders but merge them into a single receiver at
        // the task end.
        let channel = Box::new(user_recv.map(ComponentMessage::Update).select(sys_recv));

        let scope = match parent_scope {
            Some(ref p) => p.inherit(user_send),
            None => Scope::new(user_send),
        };
        let state = C::create(props);
        let initial_view = state.view();
        let ui_state = State::build(&initial_view, parent, &scope);
        (
            scope.clone(),
            sys_send,
            ComponentTask {
                scope,
                parent_scope: parent_scope.cloned(),
                state,
                ui_state,
                channel,
            },
        )
    }

    pub fn widget(&self) -> Widget {
        self.ui_state.object().clone()
    }
}

impl<C, P> Future for ComponentTask<C, P>
where
    C: 'static + Component + View<C>,
    P: Component + View<P>,
{
    type Item = ();
    type Error = Never;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let mut render = false;
        loop {
            match self.channel.poll_next(ctx) {
                Ok(Async::Ready(Some(msg))) => match msg {
                    ComponentMessage::Update(msg) => {
                        if self.state.update(msg) {
                            render = true;
                        }
                    }
                    ComponentMessage::Props(props) => {
                        if self.state.change(props) {
                            render = true;
                        }
                    }
                    ComponentMessage::Mounted => {
                        self.state.mounted();
                    }
                    ComponentMessage::Unmounted => {
                        self.state.unmounted();
                    }
                },
                Ok(Async::Pending) if render => {
                    // we patch
                    let new_view = self.state.view();
                    self.scope.mute();
                    if !self.ui_state.patch(&new_view, None, &self.scope) {
                        unimplemented!("don't know how to propagate failed patch");
                    }
                    self.scope.unmute();
                    return Ok(Async::Pending);
                }
                Ok(Async::Ready(None)) => return Ok(Async::Ready(())),
                Ok(Async::Pending) => return Ok(Async::Pending),
                Err(e) => return Err(e),
            }
        }
    }
}
