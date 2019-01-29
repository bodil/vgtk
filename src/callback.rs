use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

pub struct Callback<A>(Rc<Fn(A)>);

impl<A: Debug> Callback<A> {
    pub fn send(&self, value: A) {
        (self.0)(value)
    }
}

impl<A: Debug + 'static> Callback<A> {
    pub fn comap<F, B>(self, f: F) -> Callback<B>
    where
        F: Fn(B) -> A + 'static,
    {
        Callback::from(move |input| {
            let output = f(input);
            self.clone().send(output);
        })
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

impl<A, F: Fn(A) + 'static> From<F> for Callback<A> {
    fn from(func: F) -> Self {
        Callback(Rc::new(func))
    }
}
