use glib::futures::task::Context;
use gtk::Container;

use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::callback::Callback;
use crate::component::Component;
use crate::scope::Scope;
use crate::vdom::ComponentState;

pub struct AnyProps {
    valid: AtomicBool,
    type_id: TypeId,
    data: *mut (),
}

impl AnyProps {
    fn null() -> Self {
        AnyProps {
            valid: AtomicBool::new(false),
            type_id: TypeId::of::<()>(),
            data: std::ptr::null_mut(),
        }
    }

    pub fn new<Props: Any>(props: Props) -> Self {
        AnyProps {
            valid: AtomicBool::new(true),
            type_id: TypeId::of::<Props>(),
            data: Box::into_raw(Box::new(props)) as *mut (),
        }
    }

    pub fn unwrap<Props: Any>(&self) -> Props {
        if !self.valid.swap(false, Ordering::SeqCst) {
            panic!("tried to unwrap AnyProps of type {:?} twice", self.type_id)
        }
        if self.type_id != TypeId::of::<Props>() {
            panic!(
                "passed type {:?} to constructor expecting type {:?}",
                self.type_id,
                TypeId::of::<Props>()
            )
        }
        unsafe { *Box::from_raw(self.data as *mut Props) }
    }
}

type Constructor<Model> = Fn(&AnyProps, Option<&Container>, &Scope<Model>) -> ComponentState<Model>;

pub struct VComponent<Model: Component> {
    parent: PhantomData<Model>,
    pub model_type: TypeId,
    pub props: AnyProps,
    pub constructor: Box<Constructor<Model>>,
}

impl<Model: 'static + Component> VComponent<Model> {
    pub fn new<Child: 'static + Component>() -> Self {
        let constructor: Box<Constructor<Model>> = Box::new(ComponentState::build::<Child>);
        VComponent {
            parent: PhantomData,
            model_type: TypeId::of::<Child>(),
            props: AnyProps::null(),
            constructor,
        }
    }

    pub fn set_props<Child: 'static + Component>(&mut self, props: Child::Properties) {
        assert_eq!(self.model_type, TypeId::of::<Child>());
        self.props = AnyProps::new(props);
    }
}

pub trait PropTransform<Model: Component, From, To> {
    fn transform(&self, from: From) -> To;
}

impl<Model: Component, A> PropTransform<Model, A, A> for VComponent<Model> {
    fn transform(&self, from: A) -> A {
        from
    }
}

impl<'a, Model: Component, A: Clone> PropTransform<Model, &'a A, A> for VComponent<Model> {
    fn transform(&self, from: &'a A) -> A {
        from.clone()
    }
}

impl<'a, Model: Component> PropTransform<Model, &'a str, String> for VComponent<Model> {
    fn transform(&self, from: &'a str) -> String {
        from.to_string()
    }
}

impl<Model, F, A> PropTransform<Model, F, Option<Callback<A>>> for VComponent<Model>
where
    Model: Component + 'static,
    F: Fn(A) -> Model::Message + 'static,
{
    fn transform(&self, from: F) -> Option<Callback<A>> {
        let callback: Rc<Fn(&mut Context, A)> = Rc::new(move |ctx, arg| {
            let msg = from(arg);
            let scope = Scope::<Model>::current_parent(ctx);
            scope.send_message(msg);
        });
        Some(Callback(callback))
    }
}
