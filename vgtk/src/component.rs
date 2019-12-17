use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future::FutureExt,
    stream::{select, Stream},
    task::{Context, Poll},
    StreamExt,
};
use glib::{Cast, MainContext, Object, ObjectExt, WeakRef};
use gtk::{Application, GtkApplicationExt, Widget, WidgetExt, Window};

use std::any::TypeId;
use std::fmt::{Debug, Error, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::RwLock;

use colored::Colorize;
use log::{debug, trace};

use crate::scope::{AnyScope, Scope};
use crate::vdom::State;
use crate::vnode::VNode;

/// An action resulting from a `Component::update()`.
pub enum UpdateAction<C: Component> {
    /// No action is necessary.
    None,
    /// Re-render the widget tree.
    Render,
    /// Run an async task and update again when it completes.
    ///
    /// You should use `UpdateAction::defer()` to construct this, rather than
    /// trying to box up your `Future` yourself.
    Defer(Pin<Box<dyn Future<Output = C::Message> + 'static>>),
}

impl<C: Component> UpdateAction<C> {
    /// Construct a deferred action given a `Future`.
    pub fn defer(job: impl Future<Output = C::Message> + 'static) -> Self {
        UpdateAction::Defer(job.boxed_local())
    }
}

/// This is the trait your UI components should implement.
pub trait Component: Default + Unpin {
    /// The type of messages you can send to the `Component::update()` function.
    type Message: Clone + Send + Debug + Unpin;
    /// A struct type which holds the properties for your `Component`.
    ///
    /// The `gtk!` macro will construct this from the attributes on the
    /// corresponding component element.
    ///
    /// This is not relevant and should be set to `()` if you're writing a top
    /// level component.
    type Properties: Clone + Default;

    /// Process a `Component::Message` and update the state accordingly.
    ///
    /// If you've made changes which should be reflected in the UI state, return
    /// `UpdateAction::Render`. This will call `Component::view()` and update
    /// the widget tree accordingly.
    ///
    /// If you need to perform I/O, you can return `UpdateAction::Defer`, which
    /// will run an async action and call `Component::update()` again with its
    /// result.
    ///
    /// Otherwise, return `UpdateAction::None`.
    fn update(&mut self, _msg: Self::Message) -> UpdateAction<Self> {
        UpdateAction::None
    }

    /// Construct a new `Component` given a `Component::Properties` object.
    ///
    /// This will never be called to construct a top level component. These will
    /// always be constructed using `Default::default()`.
    fn create(_props: Self::Properties) -> Self {
        Self::default()
    }

    /// Update a `Component`'s properties.
    fn change(&mut self, _props: Self::Properties) -> UpdateAction<Self> {
        unimplemented!("add a Component::change() implementation")
    }

    /// This method is called when the `Component` becomes visible to the user.
    fn mounted(&mut self) {}

    /// This method is called when the `Component` becomes hidden or is
    /// removed entirely.
    fn unmounted(&mut self) {}

    /// Build a `VNode` tree to represent your UI.
    ///
    /// This is called whenever the `Component` needs to re-render, and its UI
    /// state will be updated to reflect the `VNode` tree.
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ComponentMessage::Update(msg) => write!(
                f,
                "{}",
                format!(
                    "ComponentMessage::Update({})",
                    format!("{:?}", msg).bright_white().bold()
                )
                .green()
            ),
            ComponentMessage::Props(_) => write!(f, "{}", "ComponentMessage::Props(...)".green()),
            ComponentMessage::Mounted => write!(f, "{}", "ComponentMessage::Mounted".green()),
            ComponentMessage::Unmounted => write!(f, "{}", "ComponentMessage::Unmounted".green()),
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

    pub(crate) fn object(&self) -> Object {
        self.task.ui_state.as_ref().unwrap().object().clone()
    }
}

pub(crate) struct ComponentTask<C, P>
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

    fn run_job(&self, job: impl Future<Output = C::Message> + 'static) {
        let scope = self.scope.clone();
        MainContext::ref_thread_default().spawn_local(async move {
            scope.send_message(job.await);
        })
    }

    pub(crate) fn process(&mut self, ctx: &mut Context<'_>) -> Poll<()> {
        let mut render = false;
        loop {
            let next = Stream::poll_next(self.channel.as_mut(), ctx);
            trace!(
                "{} {}",
                self.scope.name().bright_black(),
                format!("{:?}", next).bright_black().bold()
            );
            match next {
                Poll::Ready(Some(msg)) => match msg {
                    ComponentMessage::Update(msg) => match self.state.update(msg) {
                        UpdateAction::Defer(job) => {
                            self.run_job(job);
                        }
                        UpdateAction::Render => {
                            render = true;
                        }
                        UpdateAction::None => {}
                    },
                    ComponentMessage::Props(props) => match self.state.change(props) {
                        UpdateAction::Defer(job) => {
                            self.run_job(job);
                        }
                        UpdateAction::Render => {
                            render = true;
                        }
                        UpdateAction::None => {}
                    },
                    ComponentMessage::Mounted => {
                        debug!(
                            "{} {}",
                            "Component mounted:".bright_blue(),
                            self.scope.name().magenta().bold()
                        );
                        self.state.mounted();
                    }
                    ComponentMessage::Unmounted => {
                        if let Some(state) = self.ui_state.take() {
                            state.unmount();
                        }
                        self.state.unmounted();
                        debug!(
                            "{} {}",
                            "Component unmounted:".bright_red(),
                            self.scope.name().magenta().bold()
                        );
                        return Poll::Ready(());
                    }
                },
                Poll::Pending if render => {
                    if let Some(ref mut ui_state) = self.ui_state {
                        // we patch
                        let new_view = self.state.view();
                        self.scope.mute();
                        if !ui_state.patch(&new_view, None, &self.scope) {
                            unimplemented!(
                                "{}: don't know how to propagate failed patch",
                                self.scope.name()
                            );
                        }
                        self.scope.unmute();
                        return Poll::Pending;
                    } else {
                        debug!(
                            "{} {}",
                            self.scope.name().magenta().bold(),
                            "rendering in the absence of a UI state; exiting".bright_red()
                        );
                        return Poll::Ready(());
                    }
                }
                Poll::Ready(None) => {
                    debug!(
                        "{} {}",
                        self.scope.name().magenta().bold(),
                        "terminating because all channel handles dropped".bright_red()
                    );
                    return Poll::Ready(());
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    pub(crate) fn object(&self) -> Option<Object> {
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

/// Get the current `Object`.
///
/// When called from inside a `Component`, it will return the top level `Object`
/// for this component, if it currently exists.
///
/// When called from outside a `Component`'s lifecycle, you should hopefully
/// just receive a `None`, but, generally, try not to do that.
pub fn current_object() -> Option<Object> {
    LOCAL_CONTEXT.with(|key| {
        let lock = key.read().unwrap();
        lock.current_object
            .as_ref()
            .and_then(|object| object.upgrade())
    })
}

/// Get the current `Window`.
///
/// When called from inside a `Component`, it will return the `Window` to which
/// its top level `Object` is attached.
///
/// If the top level `Object` is a `Window`, it will return that.
///
/// If the top level `Object` is an `Application`, it will return that
/// `Application`'s idea of what its currently active `Window` is, as determined
/// by `Application::get_active_window()`.
///
/// If it's unable to determine what the current `Window` is, you'll get a
/// `None`.
///
/// When called from outside a `Component`'s lifecycle, you should hopefully
/// just receive a `None`, but, generally, try not to do that.
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

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
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
