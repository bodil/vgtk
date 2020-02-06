use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

/// A callback property for sub-[`Component`][Component]s.
///
/// When a subcomponent needs to communicate with its parent, you can use a `Callback`
/// to simulate a signal handler.
///
/// This is how you would declare a callback property that receives a [`String`][String] when
/// something happens inside the subcomponent. The property value should be a closure
/// which receives the [`String`][String] and returns a message for the parent component, of the
/// parent component's [`Component::Message`][Message] type. The framework will automatically take
/// care of figuring out the callback's type signature when you mount the subcomponent.
///
/// Note that you must always wrap the callback in an [`Option`][Option], because the properties
/// object is constructed using [`Default::default()`][default] before filling in the property values,
/// and it needs a safe and constructable default value for the callback. More crucially,
/// for this reason the machinery which figures out the callback's type signature is
/// implemented for `Option<Callback<_>>` but not for plain `Callback<_>`.
///
/// ```rust,no_run
/// # use vgtk::Callback;
/// struct MyComponentProperties {
///     on_message: Option<Callback<String>>,
/// }
/// ```
///
/// This is how you might provide a callback property to the above:
///
/// ```rust,no_run
/// # use vgtk::{gtk, VNode, Component, Callback};
/// # #[derive(Clone, Debug, Default)]
/// # pub struct MyComponent {
/// #     pub on_message: Option<Callback<String>>,
/// # }
/// # impl Component for MyComponent {
/// #     type Message = ();
/// #     type Properties = Self;
/// #     fn view(&self) -> VNode<Self> { todo!() }
/// # }
/// # #[derive(Clone, Debug)] enum ParentMessage { StringReceived(String) }
/// # #[derive(Default)] struct Parent;
/// # impl Component for Parent { type Message = ParentMessage; type Properties = ();
/// # fn view(&self) -> VNode<Self> { gtk! {
/// <@MyComponent on_message=|string| ParentMessage::StringReceived(string) />
/// # }}}
/// ```
///
/// [Component]: trait.Component.html
/// [Message]: trait.Component.html#associatedtype.Message
/// [default]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
/// [String]: https://doc.rust-lang.org/std/string/struct.String.html
/// [Option]: https://doc.rust-lang.org/std/option/enum.Option.html
pub struct Callback<A>(pub(crate) Rc<dyn Fn(A)>);

impl<A: Debug> Callback<A> {
    /// Send a value to the callback.
    pub fn send(&self, value: A) {
        (self.0)(value)
    }
}

impl<A> Clone for Callback<A> {
    fn clone(&self) -> Self {
        Callback(self.0.clone())
    }
}

impl<A> PartialEq for Callback<A> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<A> Debug for Callback<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Callback()")
    }
}

impl<A, F: Fn(A) + 'static> From<F> for Callback<A> {
    fn from(func: F) -> Self {
        Callback(Rc::new(func))
    }
}
