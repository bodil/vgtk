use std::fmt::Debug;

use crate::vitem::VItem;

pub trait Component: Default {
    type Message: Clone + Send + Debug;
    type Properties: Clone + Default;
    fn update(&mut self, msg: Self::Message) -> bool;

    fn create(_props: Self::Properties) -> Self {
        Self::default()
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        unimplemented!()
    }

    fn mounted(&mut self) {}

    fn unmounted(&mut self) {}
}

pub trait View<Model: Component> {
    fn view(&self) -> VItem<Model>;
}
