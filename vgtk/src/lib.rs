//! A declarative UI framework built on [GTK] and [Gtk-rs].
//!
//! ## Overview
//!
//! `vgtk` is a GUI framework built on [GTK] using what might be
//! called the "Model-View-Update" pattern, as popularised in [Elm]
//! and [Redux], in addition to a component model similar to [React].
//! Its primary inspiration is the [Yew] web framework for Rust, from
//! which it inherits most of its more specific ideas.
//!
//! To facilitate writing GTK UIs in a declarative style, `vgtk` implements
//! an algorithm similar to DOM diffing, but for GTK's widget tree, which
//! has turned out to be considerably less trivial than diffing a well structured
//! tree like the DOM, but as a first draft at least it gets the job done.
//!
//! More importantly, `vgtk` also provides the [`gtk!`][vgtk::gtk!] macro
//! allowing you to write your declarative UI in a syntax very similar to [JSX].
//!
//! ## Show Me!
//!
//! ```rust,no_run
//! use vgtk::{ext::*, gtk, run, Component, UpdateAction, VNode};
//! use vgtk::lib::{gtk::*, gio::ApplicationFlags};
//!
//! #[derive(Clone, Default, Debug)]
//! struct Model {
//!      counter: usize,
//! }
//!
//! #[derive(Clone, Debug)]
//! enum Message {
//!     Inc,
//!     Exit,
//! }
//!
//! impl Component for Model {
//!     type Message = Message;
//!     type Properties = ();
//!
//!     fn update(&mut self, message: Message) -> UpdateAction<Self> {
//!         match message {
//!             Message::Inc => {
//!                 self.counter += 1;
//!                 UpdateAction::Render
//!             }
//!             Message::Exit => {
//!                 vgtk::quit();
//!                 UpdateAction::None
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> VNode<Model> {
//!         gtk! {
//!             <Application::new_unwrap(None, ApplicationFlags::empty())>
//!                 <Window border_width=20 on destroy=|_| Message::Exit>
//!                     <HeaderBar title="inc!" show_close_button=true />
//!                     <Box spacing=10 halign=Align::Center>
//!                         <Label label=self.counter.to_string() />
//!                         <Button label="inc!" image="add" always_show_image=true
//!                                 on clicked=|_| Message::Inc />
//!                     </Box>
//!                 </Window>
//!             </Application>
//!         }
//!     }
//! }
//!
//! fn main() {
//!     std::process::exit(run::<Model>());
//! }
//! ```
//!
//! ## Prerequisites
//!
//! The `vgtk` documentation assumes you already have a passing familiarity with [GTK] and
//! its [Rust bindings][Gtk-rs]. It makes little to no effort to explain how [GTK] works or
//! to catalogue which widgets are available. Please refer to the [Gtk-rs] documentation or
//! that of [GTK] proper for this.
//!
//! ## The Component Model
//!
//! The core idea of `vgtk` is the [`Component`][Component]. A component, in practical terms, is a
//! composable tree of Gtk widgets, often a window, reflecting a block of application state. You
//! can write your application as a single component, but you can also embed a component inside
//! another component, which makes sense for parts of your UI you tend to repeat, or just for
//! making an easier to use interface for a common Gtk widget type.
//!
//! Your application starts with a component that manages an [`Application`][Application] object.
//! This [`Application`][Application] in turn will have one or more [`Window`][Window]s attached
//! to it, either directly inside the component or as subcomponents. [`Window`][Window]s in turn
//! contain widget trees.
//!
//! You can think of a component as an MVC system, if that's something you're familiar with: it
//! contains some application state (the Model), a method for rendering that state into a tree of
//! GTK widgets (the View) and a method for updating that state based on external inputs like
//! user interaction (the Controller). You can also think of it as mapping almost directly to a
//! [React] component, if you're more familiar with that, even down to the way it interacts with
//! the [JSX] syntax.
//!
//! ## Building A Component
//!
//! A component in `vgtk` is something which implements the [`Component`][Component] trait,
//! providing the two crucial methods [`view`][Component::view] and [`update`][Component::update].
//! Your top level component should have a [`view`][Component::view] function which returns
//! a GTK [`Application`][Application] object, or, rather, a "virtual DOM" tree which builds one.
//!
//! The [`view`][Component::view] function's job is to examine the current state of the component
//! (usually contained within the type of the [`Component`][Component] itself) and return a UI tree
//! which reflects it. This is its only job, and however much you might be tempted to, it must not do
//! anything else, especially anything that might block the thread or cause a delayed result.
//!
//! Responding to user interaction, or other external inputs, is the job of the
//! [`update`][Component::update] function. This takes an argument of the type
//! [`Component::Message`][Component::Message] and updates the component's state according to the
//! contents of the message. This is the only place you're allowed to modify the contents of your
//! component, and every way to change it should be expressed as a message you can send to
//! your [`update`][Component::update] function.
//!
//! [`update`][Component::update] returns an [`UpdateAction`][UpdateAction] describing one of three
//! outcomes: either, [`None`][UpdateAction::None], meaning nothing significant changed as a result
//! of the message and we don't need to update the UI, or [`Render`][UpdateAction::Render], meaning
//! you made a change which should be reflected in the UI, causing the framework to call your
//! [`view`][Component::view] method and re-render the UI. Finally, you can also return
//! [`Defer`][UpdateAction::Defer] with a [`Future`][Future] in case you need to
//! do some I/O or a similar asynchronous task - the [`Future`][Future] should resolve to a
//! [`Component::Message`][Component::Message] which will be passed along to [`update`][Component::update]
//! when the [`Future`][Future] resolves.
//!
//! ## Signal Handlers
//!
//! Other than [`UpdateAction::Defer`][UpdateAction::Defer], where do these messages come from?
//! Usually, they will be triggered by user interaction with the UI. Using the [`gtk!`][vgtk::gtk!]
//! macro, you can attach signal handlers to
//! [GTK signals](https://developer.gnome.org/gobject/stable/howto-signals.html)
//! which respond to a signal by sending a message to the current component.
//!
//! For instance, a GTK [`Button`][Button] has a [`clicked`][Button::connect_clicked] signal which is
//! triggered when the user clicks on the button, as the name suggests. Looking at the
//! [`connect_clicked`][Button::connect_clicked] method, we see that it takes a single `&Self` argument,
//! representing the button being clicked. In order to listen to this signal, we attach a closure
//! with a similar function signature to the button using the `on` syntax. The closure always takes the
//! same arguments as the `connect_*` callback, but instead of returning nothing it returns a message of
//! the component's message type. This message will be passed to the component's
//! [`update`][Component::update] method by the framework.
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode, Component};
//! # use vgtk::lib::gtk::{Button, ButtonExt};
//! # #[derive(Clone, Debug)] enum Message { ButtonWasClicked }
//! # #[derive(Default)] struct Comp;
//! # impl Component for Comp { type Message = Message; type Properties = (); fn view(&self) -> VNode<Self> {
//! gtk! {
//!     <Button label="Click me" on clicked=|_| Message::ButtonWasClicked />
//! }
//! # }}
//! ```
//!
//! This will cause a `Message::ButtonWasClicked` message to be sent to your component's
//! [`update`][Component::update] function when the user clicks the button.
//!
//! Signal handlers can also be declared as `async`, which will cause the framework to wrap the handler
//! in an `async {}` block and `await` the
//! message result before passing it on to your update function. For instance, this very contrived
//! example shows a message dialog asking the user to confirm clicking the button before sending the
//! `ButtonWasClicked` message.
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode, Component};
//! # use vgtk::lib::gtk::{Button, ButtonExt, DialogFlags, MessageType, ButtonsType};
//! # #[derive(Clone, Debug)] enum Message { ButtonWasClicked }
//! # #[derive(Default)] struct Comp;
//! # impl Component for Comp { type Message = Message; type Properties = (); fn view(&self) -> VNode<Self> {
//! gtk! {
//!     <Button label="Click me" on clicked=async |_| {
//!         vgtk::message_dialog(
//!             vgtk::current_window().as_ref(),
//!             DialogFlags::MODAL, MessageType::Info, ButtonsType::Ok, true,
//!             "Please confirm that you clicked the button."
//!         ).await;
//!         Message::ButtonWasClicked
//!     } />
//! }
//! # }}
//! ```
//!
//! ## The `gtk!` Syntax
//!
//! The syntax for the [`gtk!`][vgtk::gtk!] macro is similar to [JSX], but with a number of necessary
//! extensions.
//!
//! A GTK widget (or, in fact, any GLib object, but most objects require widget children) can be
//! constructed using an element tag. Attributes on that tag correspond to `get_*` and `set_*` methods
//! on the GTK widget. Thus, to construct a GTK [`Button`][Button] calling [`set_label`][Button::set_label]
//! to set its label:
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode};
//! # use vgtk::lib::gtk::{Button, ButtonExt};
//! # fn view() -> VNode<()> {
//! gtk! {
//!     <Button label="Click me" />
//! }
//! # }
//! ```
//!
//! A GTK container is represented by an open/close element tag, with child tags representing its
//! children.
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode};
//! # use vgtk::lib::gtk::{Button, ButtonExt, Box, BoxExt, Orientation, OrientableExt};
//! # fn view() -> VNode<()> {
//! gtk! {
//!     <Box orientation=Orientation::Horizontal>
//!         <Button label="Left click" />
//!         <Button label="Right click" />
//!     </Box>
//! }
//! # }
//! ```
//!
//! If a widget has a constructor that takes arguments, you can use that constructor in place
//! of the element's tag name. This syntax should only be used in cases where a widget simply cannot be constructed
//! using properties alone, because the differ isn't able to update arguments that may have changed
//! in constructors once the widget has been instantiated. It should be reserved only for when it's
//! absolutely necessary, such as when constructing an [`Application`][Application], which doesn't
//! implement [`Buildable`][Buildable] and therefore can't be constructed in any way other than through
//! a constructor method.
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode, ext::ApplicationHelpers};
//! # use vgtk::lib::{gtk::Application, gio::ApplicationFlags};
//! # fn view() -> VNode<()> {
//! gtk! {
//!     <Application::new_unwrap(None, ApplicationFlags::empty()) />
//! }
//! # }
//! ```
//!
//! Sometimes, a widget has a property which must be set through its parent, such as a child's
//! `expand` and `fill` properties inside a [`Box`][Box]. These properties correspond to
//! `set_child_*` and `get_child_*` methods on the parent, and are represented as attributes
//! on the child with the parent's type as a namespace, like this:
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode};
//! # use vgtk::lib::gtk::{Button, ButtonExt, Box, BoxExt};
//! # fn view() -> VNode<()> {
//! gtk! {
//!     <Box>
//!         <Button label="Click me" Box::expand=true Box::fill=true />
//!     </Box>
//! }
//! # }
//! ```
//!
//! The final addition to the attribute syntax pertains to when you need to qualify an
//! ambiguous method name. For instance, a [`MenuButton`][MenuButton] implements both
//! [`WidgetExt`][WidgetExt] and [`MenuButtonExt`][MenuButtonExt], both of which contains
//! a `set_direction` method. In order to let the compiler know which one you mean, you
//! can qualify it with an `@` and the type name, like this:
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode};
//! # use vgtk::lib::gtk::{MenuButton, MenuButtonExt, WidgetExt, ArrowType, TextDirection};
//! # fn view1() -> VNode<()> { gtk!{
//! <MenuButton @MenuButtonExt::direction=ArrowType::Down />
//! # }} fn view2() -> VNode<()> { gtk! {
//! <MenuButton @WidgetExt::direction=TextDirection::Ltr />
//! # }}
//! ```
//!
//! ### Interpolation
//!
//! The `gtk!` macro's parser tries to be smart about recognising Rust expressions as attribute
//! values, but it's not perfect. If the parser chokes on some particularly complicated Rust
//! expression, you can always wrap an attribute's value in a `{}` block, as per [JSX].
//!
//! This curly bracket syntax is also used to dynamically insert child widgets into a tree.
//! You can insert a code block in place of a child widget, which should return an iterator
//! of widgets that will be appended by the macro when rendering the virtual tree.
//!
//! For instance, to dynamically generate a series of buttons, you can do this:
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode};
//! # use vgtk::lib::gtk::{Button, ButtonExt, Box, BoxExt, Orientation};
//! # fn view() -> VNode<()> {
//! gtk! {
//!     <Box>
//!         {
//!             (1..=5).map(|counter| {
//!                 gtk! { <Button label=format!("Button #{}", counter) /> }
//!             })
//!         }
//!     </Box>
//! }
//! # }
//! ```
//!
//! ## Subcomponents
//!
//! Components are designed to be composable, so you can place one component inside
//! another. The `gtk!` syntax for that looks like this:
//!
//! ```rust,ignore
//! <@Subcomponent attribute_1="hello" attribute_2=1337 />
//! ```
//!
//! The subcomponent name (prefixed by `@` to distinguish it from a GTK object) maps to
//! the type of the component, and each attribute maps directly to a property on its
//! [`Component::Properties`][Component::Properties] type. When a subcomponent is constructed,
//! the framework calls its [`create`][Component::create] method with the property object constructed
//! from its attributes as an argument.
//!
//! A subcomponent needs to implement [`create`][Component::create] and [`change`][Component::change]
//! in addition to [`update`][Component::update] and [`view`][Component::view]. The default implementations
//! of these methods will panic with a message telling you to implement them.
//!
//! Subcomponents do *not* support signal handlers, because a component is not a GTK object. You'll have
//! to use the [`Callback`][Callback] type to communicate between a subcomponent and its parent.
//!
//! This is what a very simple button subcomponent might look like:
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode, UpdateAction, Component, Callback};
//! # use vgtk::lib::gtk::{Button, ButtonExt};
//! #[derive(Clone, Debug, Default)]
//! pub struct MyButton {
//!     pub label: String,
//!     pub on_clicked: Callback<()>,
//! }
//!
//! #[derive(Clone, Debug)]
//! pub enum MyButtonMessage {
//!     Clicked
//! }
//!
//! impl Component for MyButton {
//!     type Message = MyButtonMessage;
//!     type Properties = Self;
//!
//!     fn create(props: Self) -> Self {
//!         props
//!     }
//!
//!     fn change(&mut self, props: Self) -> UpdateAction<Self> {
//!         *self = props;
//!         UpdateAction::Render
//!     }
//!
//!     fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
//!         match msg {
//!             MyButtonMessage::Clicked => {
//!                 self.on_clicked.send(());
//!             }
//!         }
//!         UpdateAction::None
//!     }
//!
//!     fn view(&self) -> VNode<Self> {
//!         gtk! {
//!             <Button label=self.label.clone() on clicked=|_| MyButtonMessage::Clicked />
//!         }
//!     }
//! }
//! ```
//!
//! Note that because this component doesn't have any state other than its properties, we
//! just make `Self::Properties` equal to `Self`, there's no need to keep two identical types
//! around for this purpose. Note also that the callback passes a value of type `()`, because
//! the `clicked` signal doesn't contain any useful information besides the fact that it's
//! being sent.
//!
//! This is how you'd use this subcomponent with a callback inside the [`view`][Component::view]
//! method of a parent component:
//!
//! ```rust,no_run
//! # use vgtk::{gtk, VNode, Component, Callback};
//! # use vgtk::lib::gtk::{Button, ButtonExt, Box, BoxExt, Orientation, Label, LabelExt};
//! # #[derive(Clone, Debug, Default)]
//! # pub struct MyButton {
//! #     pub label: String,
//! #     pub on_clicked: Callback<()>,
//! # }
//! # impl Component for MyButton {
//! #     type Message = ();
//! #     type Properties = Self;
//! #     fn view(&self) -> VNode<Self> { todo!() }
//! # }
//! # #[derive(Clone, Debug)] enum ParentMessage { ButtonClicked }
//! # #[derive(Default)] struct Parent;
//! # impl Component for Parent { type Message = ParentMessage; type Properties = ();
//! fn view(&self) -> VNode<Self> {
//!     gtk! {
//!         <Box>
//!             <Label label="Here is a button:" />
//!             <@MyButton label="Click me!" on clicked=|_| ParentMessage::ButtonClicked />
//!         </Box>
//!     }
//! }
//! # }
//! ```
//!
//! Note that the return type of the `on_clicked` callback is the message type of the parent
//! component - when the subcomponent is constructed, the parent component will wire any callback
//! up to its [`update`][Component::update] function for you automatically with a bit of `unsafe`
//! trickery, so that the subcomponent doesn't have to carry the information about what type of
//! parent component it lives within inside its type signature. It'll just work, with nary a
//! profunctor in sight.
//!
//! ## Logging
//!
//! `vgtk` uses the [`log`][log] crate for debug output. You'll need to provide your own logger for this;
//! the example projects show how to set up [`pretty_env_logger`][pretty_env_logger] for logging to the
//! standard output. To enable it, set the `RUST_LOG` environment variable to `debug` when running the
//! examples. You can also use the value `vgtk=debug` to turn on debug output only for `vgtk`, if you have
//! other components using the logging framework. At log level `debug`, it will log the component messages
//! received by your components, which can be extremely helpful when trying to track down a bug
//! in your component's interactions. At log level `trace`, you'll also get a lot of `vgtk` internal
//! information that's likely only useful if you're debugging the framework.
//!
//! ## Work In Progress
//!
//! While this framework is currently sufficiently usable that we can implement [TodoMVC] in it, there
//! are likely to be a lot of rough edges still to be uncovered. In particular, a lot of properties on
//! GTK objects don't map cleanly to `get_*` and `set_*` methods in the [Gtk-rs] mappings, as required
//! by the [`gtk!`][vgtk::gtk!] macro, which has necessitated the collection of hacks in
//! [`vgtk::ext`][vgtk::ext]. There are likely many more to be found in widgets as yet unused.
//!
//! As alluded to previously, the diffing algorithm is also complicated by the irregular structure of the
//! GTK widget tree. Not all child widgets are added through the [`Container`][Container] API, and while
//! most of the exceptions are already implemented, there will be more. There's also a lot of room yet
//! for optimisation in the diffing algorithm itself, which is currently not nearly as clever as the state
//! of the art in the DOM diffing world.
//!
//! Not to mention the documentation effort.
//!
//! In short, [pull requests](https://github.com/bodil/vgtk/pulls) are welcome.
//!
//! [GTK]: https://www.gtk.org/
//! [Gtk-rs]: https://gtk-rs.org/
//! [Elm]: https://elm-lang.org/
//! [React]: https://reactjs.org/
//! [Redux]: https://redux.js.org/
//! [Yew]: https://yew.rs/
//! [JSX]: https://reactjs.org/docs/introducing-jsx.html
//! [TodoMVC]: http://todomvc.com/
//! [log]: https://crates.io/crates/log
//! [pretty_env_logger]: https://crates.io/crates/pretty_env_logger
//! [vgtk::gtk!]: macro.gtk.html
//! [vgtk::ext]: ext/index.html
//! [Component]: trait.Component.html
//! [Component::view]: trait.Component.html#tymethod.view
//! [Component::update]: trait.Component.html#method.update
//! [Component::create]: trait.Component.html#method.create
//! [Component::change]: trait.Component.html#method.change
//! [Component::Message]: trait.Component.html#associatedtype.Message
//! [Component::Properties]: trait.Component.html#associatedtype.Properties
//! [Callback]: struct.Callback.html
//! [UpdateAction]: enum.UpdateAction.html
//! [UpdateAction::None]: enum.UpdateAction.html#variant.None
//! [UpdateAction::Render]: enum.UpdateAction.html#variant.Render
//! [UpdateAction::Defer]: enum.UpdateAction.html#variant.Defer
//! [Application]: ../gtk/struct.Application.html
//! [Buildable]: ../gtk/struct.Buildable.html
//! [Button]: ../gtk/struct.Button.html
//! [Button::connect_clicked]: ../gtk/trait.ButtonExt.html#tymethod.connect_clicked
//! [Button::set_label]: ../gtk/trait.ButtonExt.html#tymethod.set_label
//! [Box]: ../gtk/struct.Box.html
//! [Box::new]: ../gtk/struct.Box.html#method.new
//! [Container]: ../gtk/struct.Container.html
//! [MenuButton]: ../gtk/struct.MenuButton.html
//! [MenuButtonExt]: ../gtk/trait.MenuButtonExt.html
//! [WidgetExt]: ../gtk/trait.WidgetExt.html
//! [Window]: ../gtk/struct.Window.html
//! [Future]: https://doc.rust-lang.org/std/future/trait.Future.html

#![forbid(rust_2018_idioms)]
#![deny(nonstandard_style, unsafe_code)]
#![warn(unreachable_pub, missing_docs)]
#![allow(clippy::needless_doctest_main)]

mod callback;
mod component;
pub mod ext;
mod menu_builder;
#[doc(hidden)]
pub mod properties;
#[doc(hidden)]
pub mod scope;
pub mod types;
mod vdom;
#[doc(hidden)]
pub mod vnode;

use proc_macro_hack::proc_macro_hack;

/// Generate a virtual component tree.
///
/// See the [top level documentation][toplevel] for a description of its syntax.
///
/// [toplevel]: index.html
#[proc_macro_hack(support_nested)]
pub use vgtk_macros::gtk;

use gio::prelude::*;
use gio::Cancellable;
use glib::MainContext;
use gtk::prelude::*;
use gtk::{
    Application, ButtonsType, Dialog, DialogFlags, MessageDialog, MessageType, ResponseType, Window,
};

use futures::channel::oneshot::{self, Canceled};
use std::future::Future;

use colored::Colorize;
use log::debug;

use crate::component::{ComponentMessage, ComponentTask, PartialComponentTask};

pub use crate::callback::Callback;
pub use crate::component::{current_object, current_window, Component, UpdateAction};
pub use crate::menu_builder::{menu, MenuBuilder};
pub use crate::scope::Scope;
pub use crate::vnode::{VNode, VNodeIterator};

/// Re-exports of GTK and its associated libraries.
///
/// It is recommended that you use these rather than pulling them in as
/// dependencies of your own project, to avoid versioning conflicts.
pub mod lib {
    pub use ::gdk;
    pub use ::gdk_pixbuf;
    pub use ::gio;
    pub use ::glib;
    pub use ::gtk;
}

/// Run an [`Application`][Application] component until termination.
///
/// This is generally the function you'll call to get everything up and running.
/// Note that you pass your top level component as a type argument, not a value
/// argument. The framework will construct the component state automatically using
/// [`Default::default()`][default] before launching the component.
///
/// You can call [`vgtk::quit()`][quit] from inside the component or any subcomponent
/// to signal the application to terminate normally. This is equivalent to calling
/// [`Application::quit()`][Application::quit] on the [`Application`][Application]
/// object directly.
///
/// It's the equivalent of calling [`vgtk::start::<Component>()`][start] and then calling
/// [`Application::run()`][Application::run] on the returned `Application` object.
///
/// # Examples
///
/// ```rust,no_run
/// # type MyComponent = ();
/// let return_code = vgtk::run::<MyComponent>();
/// std::process::exit(return_code);
/// ```
///
/// [Application]: ../gtk/struct.Application.html
/// [default]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
/// [quit]: fn.quit.html
/// [start]: fn.start.html
/// [Application::quit]: ../gio/trait.ApplicationExt.html#tymethod.quit
/// [Application::run]: ../gio/trait.ApplicationExt.html#tymethod.run
pub fn run<C: 'static + Component>() -> i32 {
    let (app, _) = start::<C>();
    let args: Vec<String> = std::env::args().collect();
    app.run(&args)
}

/// Start an [`Application`][Application] component.
///
/// This will instantiate the component, construct the [`Application`][Application]
/// object and register it as the default [`Application`][Application]. You will need
/// to call [`Application::run()`][Application::run] on this to actually start the
/// GTK event loop and activate the application.
///
/// Calling this instead of [vgtk::run()][run] is useful if you need to get your
/// component's [`Scope`][Scope] in order to fire off some async work at startup and
/// notify it when the work is done.
///
/// # Examples
///
/// ```rust,no_run
/// # use vgtk::lib::gio::prelude::ApplicationExtManual;
/// # type MyComponent = ();
/// let (app, scope) = vgtk::start::<MyComponent>();
/// let args: Vec<String> = std::env::args().collect();
/// std::process::exit(app.run(&args));
/// ```
///
/// [Application]: ../gtk/struct.Application.html
/// [default]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
/// [quit]: fn.quit.html
/// [run]: fn.run.html
/// [Application::quit]: ../gio/trait.ApplicationExt.html#tymethod.quit
/// [Application::run]: ../gio/trait.ApplicationExt.html#tymethod.run
pub fn start<C: 'static + Component>() -> (Application, Scope<C>) {
    gtk::init().expect("GTK failed to initialise");
    let partial_task = PartialComponentTask::<C, ()>::new(Default::default(), None, None);
    let app: Application = partial_task.object().downcast().unwrap_or_else(|_| {
        panic!(
            "The top level object must be an Application, but {} was found.",
            partial_task.object().get_type()
        )
    });
    app.set_default();
    app.register(None as Option<&Cancellable>)
        .expect("unable to register Application");

    let scope = partial_task.scope();
    let const_app = app.clone();

    let constructor = once(move |_| {
        let (channel, task) = partial_task.finalise();
        MainContext::ref_thread_default().spawn_local(task);
        channel.unbounded_send(ComponentMessage::Mounted).unwrap();
        const_app.connect_shutdown(move |_| {
            channel.unbounded_send(ComponentMessage::Unmounted).unwrap();
        });
    });

    app.connect_activate(move |_| {
        debug!("{}", "Application has activated.".bright_blue());
        constructor(());
    });

    (app, scope)
}

/// Launch a [`Dialog`][Dialog] component as a modal dialog.
///
/// The parent window will be blocked until it resolves.
///
/// It returns a [`Future`][Future] which resolves either to `Ok(`[`ResponseType`][ResponseType]`)` when the
/// `response` signal is emitted, or to `Err(`[`Canceled`][Canceled]`)` if the dialog is
/// destroyed before the user responds to it.
///
/// [Dialog]: ../gtk/struct.Dialog.html
/// [ResponseType]: ../gtk/enum.ResponseType.html
/// [Future]: https://doc.rust-lang.org/std/future/trait.Future.html
/// [Canceled]: https://docs.rs/futures/latest/futures/channel/oneshot/struct.Canceled.html
pub fn run_dialog<C: 'static + Component>(
    parent: Option<&Window>,
) -> impl Future<Output = Result<ResponseType, Canceled>> {
    let (channel, task) = ComponentTask::<C, ()>::new(Default::default(), None, None);
    let dialog: Dialog = task
        .object()
        .unwrap()
        .downcast()
        .expect("Dialog must be a gtk::Dialog");
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
    }
    MainContext::ref_thread_default().spawn_local(task);
    let (notify, result) = oneshot::channel();
    channel.unbounded_send(ComponentMessage::Mounted).unwrap();
    let resolve = once(move |response| if notify.send(response).is_err() {});
    dialog.connect_response(move |_, response| {
        resolve(response);
        channel.unbounded_send(ComponentMessage::Unmounted).unwrap()
    });
    dialog.present();
    result
}

/// Turn an `FnOnce(A)` into an `Fn(A)` that will panic if you call it twice.
fn once<A, F: FnOnce(A)>(f: F) -> impl Fn(A) {
    use std::cell::Cell;
    use std::rc::Rc;

    let f = Rc::new(Cell::new(Some(f)));
    move |value| {
        if let Some(f) = f.take() {
            f(value);
        } else {
            panic!("vgtk::once() function called twice 😒");
        }
    }
}

/// Tell the running [`Application`][Application] to quit.
///
/// This calls [`Application::quit()`][Application::quit] on the current default
/// [`Application`][Application]. It
/// will cause the [`vgtk::run()`][run] in charge of that [`Application`][Application]
/// to terminate.
///
/// [Application]: ../gtk/struct.Application.html
/// [Application::quit]: ../gio/trait.ApplicationExt.html#tymethod.quit
/// [run]: fn.run.html
pub fn quit() {
    gio::Application::get_default()
        .expect("no default Application!")
        .quit();
}

/// Connect a GLib signal to a [`Future`][Future].
///
/// This macro takes a GLib object and the name of a method to connect it to a
/// signal (generally of the form `connect_signal_name`), and generates an
/// `async` block that will resolve with the emitted value the first time the
/// signal is emitted.
///
/// The output type of the async block is `Result<T, `[`Canceled`][Canceled]`>`, where `T` is
/// the type of the emitted value (the second argument to the callback
/// `connect_signal_name` takes). It will produce `Err(`[`Canceled`][Canceled]`)` if the object
/// is destroyed before the signal is emitted.
///
/// # Examples
///
/// ```rust,no_run
/// # use vgtk::on_signal;
/// # use vgtk::lib::gtk::{AboutDialog, AboutDialogExt, DialogExt, ResponseType, WidgetExt};
/// # async {
/// let dialog = AboutDialog::new();
/// dialog.set_program_name("Frobnicator");
/// dialog.show();
/// if on_signal!(dialog, connect_response).await == Ok(ResponseType::Accept) {
///     println!("Dialog accepted");
/// } else {
///     println!("Dialog not accepted");
/// }
/// # };
/// ```
///
/// [Future]: https://doc.rust-lang.org/std/future/trait.Future.html
/// [Canceled]: https://docs.rs/futures/latest/futures/channel/oneshot/struct.Canceled.html
#[macro_export]
macro_rules! on_signal {
    ($object:expr, $connect:ident) => {
        async {
            let (notify, result) = futures::channel::oneshot::channel();
            let state = std::sync::Arc::new(std::sync::Mutex::new((None, Some(notify))));
            let state_outer = state.clone();
            let id = $object.$connect(move |obj, value| {
                let mut lock = state.lock().unwrap();
                if let Some(notify) = lock.1.take() {
                    if notify.send(value).is_ok() {}
                }
                if let Some(handler) = lock.0.take() {
                    $crate::lib::glib::ObjectExt::disconnect(obj, handler);
                }
            });
            state_outer.lock().unwrap().0 = Some(id);
            result.await
        }
    };
}

/// Connect a GLib signal to a [`Stream`][Stream].
///
/// This macro takes a GLib object and the name of a method to connect it to a
/// signal (generally of the form `connect_signal_name`), and generates a
/// [`Stream`][Stream] that will produce a value every time the signal is emitted.
///
/// The output type of the stream is the type of the emitted value (the second
/// argument to the callback `connect_signal_name` takes). The stream will
/// terminate when the object it's connected to is destroyed.
///
/// Note that this only works with `connect_*` callbacks which take two
/// arguments. The second argument will be the contents of the stream. The first
/// argument, normally a reference to the signal's sender, is ignored.
///
/// # Examples
///
/// ```rust,no_run
/// # use futures::{future, stream::StreamExt};
/// # use vgtk::stream_signal;
/// # use vgtk::lib::gtk::{AboutDialog, AboutDialogExt, DialogExt, ResponseType, WidgetExt};
/// let dialog = AboutDialog::new();
/// dialog.set_program_name("Frobnicator");
/// dialog.show();
/// stream_signal!(dialog, connect_response).for_each(|response| {
///     println!("Dialog response: {:?}", response);
///     future::ready(())
/// });
/// ```
///
/// [Stream]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
#[macro_export]
macro_rules! stream_signal {
    ($object:expr, $connect:ident) => {{
        let (input, output) = futures::channel::mpsc::unbounded();
        $object.$connect(move |_, value| if input.unbounded_send(value).is_ok() {});
        output
    }};
}

/// Open a simple [`MessageDialog`][MessageDialog].
///
/// The arguments are passed directly to [`MessageDialog::new()`][new].
/// The `is_markup` flag, if set, will interpret the `message` as markup rather than plain text
/// (see [`MessageDialog::set_markup()`][set_markup]).
///
/// It returns a [`Future`][Future] which will resolve to the [`ResponseType`][ResponseType]
/// the user responds with.
///
/// # Examples
///
/// ```rust,no_run
/// # use vgtk::lib::gtk::{DialogFlags, MessageType, ButtonsType};
/// # async {
/// vgtk::message_dialog(
///     vgtk::current_window().as_ref(),
///     DialogFlags::MODAL,
///     MessageType::Error,
///     ButtonsType::OkCancel,
///     true,
///     "<b>ERROR:</b> Unknown error."
/// ).await;
/// # };
/// ```
///
/// [Future]: https://doc.rust-lang.org/std/future/trait.Future.html
/// [ResponseType]: ../gtk/enum.ResponseType.html
/// [MessageDialog]: ../gtk/struct.MessageDialog.html
/// [new]: ../gtk/struct.MessageDialog.html#method.new
/// [set_markup]: ../gtk/trait.MessageDialogExt.html#tymethod.set_markup
pub async fn message_dialog<W, S>(
    parent: Option<&W>,
    flags: DialogFlags,
    message_type: MessageType,
    buttons: ButtonsType,
    is_markup: bool,
    message: S,
) -> ResponseType
where
    W: IsA<Window>,
    S: AsRef<str>,
{
    let dialog = MessageDialog::new(parent, flags, message_type, buttons, message.as_ref());
    dialog.set_modal(true);
    if is_markup {
        dialog.set_markup(message.as_ref());
    }
    dialog.show();
    let response = on_signal!(dialog, connect_response).await;
    dialog.destroy();
    response.unwrap()
}

/// Generate a virtual component tree only if a condition is true.
///
/// You'll very often want to insert a widget only if a certain condition is true,
/// and insert nothing at all otherwise. This macro automates this common pattern.
/// It will validate your condition, and if true, it will return a [`VNodeIterator`][VNodeIterator]
/// containing the widget tree you specify. If false, it will use [`VNode::empty()`][VNode::empty]
/// to make an empty iterator.
///
/// # Examples
///
/// ```rust,no_run
/// # use vgtk::lib::gtk::{Button, ButtonExt, Box};
/// # use vgtk::{gtk, gtk_if, VNode};
/// # fn view() -> VNode<()> {
/// let buttons = 2;
/// gtk! {
///     <Box>
///         <Button label="Button 1" />
///         {
///             gtk_if!(buttons == 2 => {
///                 <Button label="Button 2" />
///             })
///         }
///     </Box>
/// }
/// # }
/// ```
///
/// This generates code equivalent to the following, which is how you'd do it
/// without the macro:
///
/// ```rust,no_run
/// # use vgtk::lib::gtk::{Button, ButtonExt, Box};
/// # use vgtk::{gtk, gtk_if, VNode};
/// # fn view() -> VNode<()> {
/// let buttons = 2;
/// gtk! {
///     <Box>
///         <Button label="Button 1" />
///         {
///             if buttons == 2 {
///                 gtk!(<Button label="Button 2" />).into_iter()
///             } else {
///                 VNode::empty()
///             }
///         }
///     </Box>
/// }
/// # }
/// ```
///
/// [VNodeIterator]: struct.VNodeIterator.html
/// [VNode::empty]: enum.VNode.html#method.empty
#[macro_export]
macro_rules! gtk_if {
    ($cond:expr => $body:tt ) => {
        if $cond {
            (gtk! $body).into_iter()
        } else {
            $crate::VNode::empty()
        }
    }
}
