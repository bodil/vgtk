use glib::futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    task::Context,
    Async, Future, Never, Poll, Stream, StreamExt,
};
use gtk::Container;
use gtk::Widget;

use std::fmt::Debug;

use crate::scope::Scope;
use crate::vdom::State;
use crate::vitem::VItem;

pub trait Component: Default {
    type Message: Clone + Send + Debug;
    type Properties: Clone + Default;
    fn update(&mut self, msg: Self::Message) -> bool;

    fn create(_props: Self::Properties) -> Self {
        Self::default()
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        unimplemented!()
    }

    fn mounted(&mut self) {}

    fn unmounted(&mut self) {}

    fn view(&self) -> VItem<Self>;
}

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

pub struct ComponentTask<C, P>
where
    C: Component,
    P: Component,
{
    scope: Scope<C>,
    parent_scope: Option<Scope<P>>,
    state: C,
    ui_state: State<C>,
    channel: Box<Stream<Item = ComponentMessage<C>, Error = Never>>,
}

impl<C, P> ComponentTask<C, P>
where
    C: 'static + Component,
    P: Component,
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
    C: 'static + Component,
    P: Component,
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
