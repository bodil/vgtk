use glib::futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    stream::{select, Stream},
    task::Context,
    Future, Poll, StreamExt,
};
use glib::{Cast, Object, ObjectExt, WeakRef};
use gtk::{Application, GtkApplicationExt, Widget, WidgetExt, Window};

use std::any::TypeId;
use std::fmt::{Debug, Error, Formatter};
use std::pin::Pin;
use std::sync::RwLock;

use log::{debug, trace};

use crate::scope::{AnyScope, Scope};
use crate::vdom::State;
use crate::vnode::VNode;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UpdateAction {
    None,
    Render,
}

pub trait Component: Default + Unpin {
    type Message: Clone + Send + Debug;
    type Properties: Clone + Default;

    fn update(&mut self, _msg: Self::Message) -> UpdateAction {
        UpdateAction::None
    }

    fn create(_props: Self::Properties) -> Self {
        Self::default()
    }

    fn change(&mut self, _props: Self::Properties) -> UpdateAction {
        unimplemented!("add a Component::change() implementation")
    }

    fn mounted(&mut self) {}

    fn unmounted(&mut self) {}

    fn view(&self) -> VNode<Self>;
}

impl Component for () {
    type Message = ();
    type Properties = ();
    fn view(&self) -> VNode<Self> {
        unimplemented!("tried to render a null component")
    }
}

pub(crate) enum ComponentMessage<C: Component> {
    Update(C::Message),
    Props(C::Properties),
    Mounted,
    Unmounted,
}

impl<C: Component> Debug for ComponentMessage<C> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            ComponentMessage::Update(msg) => {
                write!(f, "ComponentMessage::Update(")?;
                msg.fmt(f)?;
                write!(f, ")")
            }
            ComponentMessage::Props(_) => write!(f, "ComponentMessage::Props(...)"),
            ComponentMessage::Mounted => write!(f, "ComponentMessage::Mounted"),
            ComponentMessage::Unmounted => write!(f, "ComponentMessage::Unmounted"),
        }
    }
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

pub(crate) struct PartialComponentTask<C, P>
where
    C: Component,
    P: Component,
{
    task: ComponentTask<C, P>,
    view: VNode<C>,
    sender: UnboundedSender<ComponentMessage<C>>,
}

impl<C, P> PartialComponentTask<C, P>
where
    C: 'static + Component,
    P: 'static + Component,
{
    /// Start building a `ComponentTask` by initialising the task and the root
    /// object but not the children.
    ///
    /// This is generally only useful when you're constructing an `Application`,
    /// where windows should not be added to it until it's been activated, but
    /// you need to have the `Application` object in order to activate it.
    pub(crate) fn new(
        props: C::Properties,
        parent: Option<&Object>,
        parent_scope: Option<&Scope<P>>,
    ) -> Self {
        let (sys_send, sys_recv) = unbounded();
        let (user_send, user_recv) = unbounded();

        // As `C::Message` must be `Send` but `C::Properties` can't be,
        // we keep two senders but merge them into a single receiver at
        // the task end.
        let channel = Pin::new(Box::new(select(
            user_recv.map(ComponentMessage::Update),
            sys_recv,
        )));

        let type_name = std::any::type_name::<C>();
        let scope = match parent_scope {
            Some(ref p) => p.inherit(type_name, user_send),
            None => Scope::new(type_name, user_send),
        };
        let state = C::create(props);
        let initial_view = state.view();
        let ui_state = State::build_root(&initial_view, parent, &scope);
        PartialComponentTask {
            task: ComponentTask {
                scope,
                parent_scope: parent_scope.cloned(),
                state,
                ui_state: Some(ui_state),
                channel,
            },
            view: initial_view,
            sender: sys_send,
        }
    }

    /// Finalise the partially constructed `ComponentTask` by constructing its
    /// children.
    pub(crate) fn finalise(
        mut self,
    ) -> (UnboundedSender<ComponentMessage<C>>, ComponentTask<C, P>) {
        if let Some(ref mut ui_state) = self.task.ui_state {
            ui_state.build_children(&self.view, &self.task.scope);
        }
        (self.sender, self.task)
    }

    pub fn object(&self) -> Object {
        self.task.ui_state.as_ref().unwrap().object().clone()
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
    ui_state: Option<State<C>>,
    channel: Pin<Box<dyn Stream<Item = ComponentMessage<C>>>>,
}

impl<C, P> ComponentTask<C, P>
where
    C: 'static + Component,
    P: 'static + Component,
{
    pub(crate) fn new(
        props: C::Properties,
        parent: Option<&Object>,
        parent_scope: Option<&Scope<P>>,
    ) -> (UnboundedSender<ComponentMessage<C>>, Self) {
        PartialComponentTask::new(props, parent, parent_scope).finalise()
    }

    pub fn process(&mut self, ctx: &mut Context) -> Poll<()> {
        let mut render = false;
        loop {
            let next = Stream::poll_next(self.channel.as_mut(), ctx);
            trace!("{}: {:?}", self.scope.name(), next);
            match next {
                Poll::Ready(Some(msg)) => match msg {
                    ComponentMessage::Update(msg) => {
                        if self.state.update(msg) == UpdateAction::Render {
                            render = true;
                        }
                    }
                    ComponentMessage::Props(props) => {
                        if self.state.change(props) == UpdateAction::Render {
                            render = true;
                        }
                    }
                    ComponentMessage::Mounted => {
                        debug!("Component mounted: {}", self.scope.name());
                        self.state.mounted();
                    }
                    ComponentMessage::Unmounted => {
                        if let Some(state) = self.ui_state.take() {
                            state.unmount();
                        }
                        self.state.unmounted();
                        debug!("Component unmounted: {}", self.scope.name());
                        return Poll::Ready(());
                    }
                },
                Poll::Pending if render => {
                    if let Some(ref mut ui_state) = self.ui_state {
                        // we patch
                        let new_view = self.state.view();
                        self.scope.mute();
                        trace!("{}: patching", self.scope.name());
                        if !ui_state.patch(&new_view, None, &self.scope) {
                            trace!("{}: patch failed!", self.scope.name());
                            unimplemented!(
                                "{}: don't know how to propagate failed patch",
                                self.scope.name()
                            );
                        }
                        self.scope.unmute();
                        return Poll::Pending;
                    } else {
                        debug!(
                            "component {} rendering in the absence of a UI state; exiting",
                            self.scope.name()
                        );
                        return Poll::Ready(());
                    }
                }
                Poll::Ready(None) => {
                    debug!(
                        "Component {} terminating because all channel handles dropped",
                        self.scope.name()
                    );
                    return Poll::Ready(());
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    pub fn object(&self) -> Option<Object> {
        self.ui_state.as_ref().map(|state| state.object().clone())
    }

    pub(crate) fn current_parent_scope() -> Scope<C> {
        LOCAL_CONTEXT.with(|key| {
            let lock = key.read().unwrap();
            match &lock.parent_scope {
                None => panic!("current task has no parent scope set!"),
                Some(any_scope) => match any_scope.try_get::<C>() {
                    None => panic!(
                        "unexpected type for current parent scope (expected {:?})",
                        TypeId::of::<C::Properties>()
                    ),
                    Some(scope) => scope.clone(),
                },
            }
        })
    }
}

pub fn current_object() -> Option<Object> {
    LOCAL_CONTEXT.with(|key| {
        let lock = key.read().unwrap();
        lock.current_object
            .as_ref()
            .and_then(|object| object.upgrade())
    })
}

pub fn current_window() -> Option<Window> {
    current_object().and_then(|obj| match obj.downcast::<Window>() {
        Ok(window) => Some(window),
        Err(obj) => match obj.downcast::<Application>() {
            Ok(app) => app.get_active_window(),
            Err(obj) => match obj.downcast::<Widget>() {
                Ok(widget) => widget
                    .get_toplevel()
                    .and_then(|toplevel| toplevel.downcast::<Window>().ok()),
                _ => None,
            },
        },
    })
}

#[derive(Default)]
struct LocalContext {
    parent_scope: Option<AnyScope>,
    current_object: Option<WeakRef<Object>>,
}

thread_local! {
    static LOCAL_CONTEXT: RwLock<LocalContext> = RwLock::new(Default::default())
}

impl<C, P> Future for ComponentTask<C, P>
where
    C: 'static + Component,
    P: 'static + Component,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        LOCAL_CONTEXT.with(|key| {
            *key.write().unwrap() = LocalContext {
                parent_scope: self.parent_scope.as_ref().map(|scope| scope.clone().into()),
                current_object: self
                    .ui_state
                    .as_ref()
                    .map(|state| state.object().downgrade()),
            };
        });
        let polled = self.get_mut().process(ctx);
        LOCAL_CONTEXT.with(|key| {
            *key.write().unwrap() = Default::default();
        });
        polled
    }
}
