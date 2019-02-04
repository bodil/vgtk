use glib::futures::task::Context;

use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

pub struct Callback<A>(pub(crate) Rc<Fn(&mut Context, A)>);

impl<A: Debug> Callback<A> {
    pub fn send(&self, context: &mut Context, value: A) {
        (self.0)(context, value)
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Callback()")
    }
}

impl<A, F: Fn(&mut Context, A) + 'static> From<F> for Callback<A> {
    fn from(func: F) -> Self {
        Callback(Rc::new(func))
    }
}
