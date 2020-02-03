use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

/// A callback property for sub-`Component`s.
///
/// When a subcomponent needs to communicate with its parent, you can use a `Callback`
/// to simulate a signal handler.
///
/// This is how you would declare a callback property that receives a `String` when
/// something happens inside the subcomponent. The property value should be a closure
/// which receives the `String` and returns a message for the parent component, of the
/// parent component's `Component::Message` type. The framework will automatically take
/// care of figuring out the callback's type signature when you mount the subcomponent.
///
/// Note that you must always wrap the callback in an `Option`, because the properties
/// object is constructed using `Default::default()` before filling in the property values,
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
/// ```rust,ignore
/// <@MyComponent on_message=|string| ParentMessage::StringReceived(string) />
/// ```
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
