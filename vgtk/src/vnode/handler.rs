use glib::{signal::SignalHandlerId, Object};

use crate::{Component, Scope};

pub struct VHandler<Model: Component> {
    pub name: &'static str,
    pub id: &'static str,
    pub set: Box<dyn Fn(&Object, &Scope<Model>) -> SignalHandlerId>,
}
