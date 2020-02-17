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
/// Note that the [`Default`][Default] implementation for `Callback` constructs an empty callback,
/// which does nothing and allocates nothing. This is the desired behaviour when using a callback
/// as a [`Component`][Component] property: if the user doesn't specify a callback explicitly, there
/// shouldn't be a callback.
///
/// ```rust,no_run
/// # use vgtk::Callback;
/// struct MyComponentProperties {
///     on_message: Callback<String>,
/// }
/// ```
///
/// This is how you might provide a callback property to the above:
///
/// ```rust,no_run
/// # use vgtk::{gtk, VNode, Component, Callback};
/// # #[derive(Clone, Debug, Default)]
/// # pub struct MyComponent {
/// #     pub on_message: Callback<String>,
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
/// [Default]: https://doc.rust-lang.org/std/default/trait.Default.html
/// [String]: https://doc.rust-lang.org/std/string/struct.String.html
/// [Option]: https://doc.rust-lang.org/std/option/enum.Option.html
pub struct Callback<A>(pub(crate) Option<Rc<dyn Fn(A)>>);

impl<A> Callback<A> {
    /// Send a value to the callback.
    ///
    /// If the callback is empty, this has no effect.
    pub fn send(&self, value: A) {
        if let Some(callback) = &self.0 {
            callback(value)
        }
    }

    /// Test whether a callback is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl<A> Default for Callback<A> {
    fn default() -> Self {
        Callback(None)
    }
}

impl<A> Clone for Callback<A> {
    fn clone(&self) -> Self {
        Callback(self.0.clone())
    }
}

impl<A> PartialEq for Callback<A> {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(left), Some(right)) = (&self.0, &other.0) {
            Rc::ptr_eq(left, right)
        } else {
            false
        }
    }
}

impl<A> Debug for Callback<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Callback()")
    }
}

impl<A, F: Fn(A) + 'static> From<F> for Callback<A> {
    fn from(func: F) -> Self {
        Callback(Some(Rc::new(func)))
    }
}
