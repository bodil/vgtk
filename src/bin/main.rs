extern crate gio;
extern crate glib;
extern crate gtk;

#[macro_use]
extern crate vgtk;

use gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{Button, ButtonsType, DialogFlags, Entry, Grid, MessageDialog, MessageType, Window};
use vgtk::{Application, Component, Event, VObject, View};

struct Model {
    dog: i32,
}

enum Msg {
    NoOp,
    UpDog,
}

impl Component for Model {
    type Message = Msg;

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NoOp => false,
            Msg::UpDog => {
                self.dog += 1;
                true
            }
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Model { dog: 0 }
    }
}

impl View<Model> for Model {
    fn view(&self) -> VObject<Model> {
        gtk!{
            <Window title="Updog",>
                <Grid column_spacing=10, row_spacing=10,>
                    <Entry text=format!("{}", self.dog), left_attach=0, top_attach=0, width=2, />
                    <Button label="What's Updog?", left_attach=0,top_attach=1, on clicked=not_much, />
                    <Button label="Up the dog", left_attach=1, top_attach=1, on clicked=|_| Msg::UpDog, />
                </Grid>
            </Window>
        }
    }
}

fn not_much(e: Event) -> Msg {
    let button: Button = e.source.downcast().unwrap();
    let window = button
        .get_toplevel()
        .and_then(|w| w.downcast::<Window>().ok());
    let dialog = MessageDialog::new(
        window.as_ref(),
        DialogFlags::DESTROY_WITH_PARENT | DialogFlags::USE_HEADER_BAR,
        MessageType::Info,
        ButtonsType::Close,
        "Not much, dog, what's up with you?",
    );
    dialog.connect_response(|d, _| d.destroy());
    dialog.run();
    Msg::NoOp
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    ::std::process::exit(Application::<Model>::run(
        "camp.lol.updog",
        ApplicationFlags::empty(),
        &args,
    ));
}
