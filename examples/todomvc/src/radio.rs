use std::fmt::{Debug, Display};

use strum::IntoEnumIterator;

use vgtk::lib::gtk::prelude::*;
use vgtk::lib::gtk::*;
use vgtk::{gtk, Callback, Component, UpdateAction, VNode};

#[derive(Clone, Debug, Default)]
pub struct Radio<Enum: Unpin> {
    pub active: Enum,
    pub on_changed: Callback<Enum>,
}

#[derive(Clone, Debug)]
pub enum RadioMsg<Enum: Unpin> {
    Selected(Enum),
}

impl<Enum, I> Component for Radio<Enum>
where
    Enum: 'static
        + IntoEnumIterator<Iterator = I>
        + Display
        + PartialEq
        + Debug
        + Default
        + Copy
        + Send
        + Unpin,
    I: Iterator<Item = Enum>,
{
    type Message = RadioMsg<Enum>;
    type Properties = Self;

    fn create(props: Self::Properties) -> Self {
        props
    }

    fn change(&mut self, props: Self::Properties) -> UpdateAction<Self> {
        *self = props;
        UpdateAction::Render
    }

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        match msg {
            RadioMsg::Selected(selected) => {
                self.active = selected;
                self.on_changed.send(self.active);
            }
        }
        UpdateAction::Render
    }

    fn view(&self) -> VNode<Self> {
        gtk! {
            <Box orientation=Orientation::Horizontal spacing=10>
                { Enum::iter().map(|label| {
                    gtk!{
                        <ToggleButton label=label.to_string() active=label == self.active
                                      on toggled=|_| RadioMsg::Selected(label)/>
                    }
                }) }
            </Box>
        }
    }
}
