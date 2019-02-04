#![recursion_limit = "16384"]
#![allow(clippy::cyclomatic_complexity)]

extern crate gio;
extern crate glib;
extern crate gtk;

#[macro_use]
extern crate vgtk;

use gio::ApplicationFlags;
use glib::futures::task::Context;
use gtk::prelude::*;
use gtk::*;
use vgtk::{run, Component, VItem};

#[derive(Clone, Debug, Default)]
struct Model {
    counter: usize,
}

#[derive(Clone, Debug)]
enum Message {
    Inc,
    Exit,
}

impl Component for Model {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, _ctx: &mut Context, msg: Self::Message) -> bool {
        match msg {
            Message::Inc => {
                self.counter += 1;
                true
            }
            Message::Exit => {
                vgtk::main_quit(0);
                false
            }
        }
    }

    fn view(&self) -> VItem<Model> {
        gtk! {
            <Window border_width=20u32,
                    on destroy=|_| Message::Exit,>
                <HeaderBar title="inc!", subtitle="AD ASTRA AD INFINITVM",
                           show_close_button=true, />
                <Box spacing=10, halign=Align::Center,>
                    <Label label=self.counter.to_string(),/>
                    <Button label="inc!", on clicked=|_| Message::Inc,/>
                </Box>
            </Window>
        }
    }
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    std::process::exit(run::<Model>(
        "camp.lol.updog",
        ApplicationFlags::empty(),
        &args,
    ));
}
