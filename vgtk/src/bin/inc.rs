use gio::ApplicationFlags;
use glib::futures::task::Context;
use gtk::{prelude::*, Align, Box, Button, HeaderBar, Label, Window};
use vgtk::{go, gtk, vnode::VNode, Component};

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

    fn view(&self) -> VNode<Model> {
        gtk! {
            <Window border_width=20u32
                    on destroy=|_| {Message::Exit}>
                <HeaderBar title="inc!" subtitle="AD ASTRA AD INFINITVM"
                           show_close_button=true />
                <Box spacing=10 halign={Align::Center}>
                    { gtk! {
                        <Label label={self.counter.to_string()} />
                    } }
                    <Button label="inc!" on clicked=|_| {Message::Inc} />
                </Box>
            </Window>
        }
    }
}

fn main() {
    std::process::exit(go::<Model>("camp.lol.updog", ApplicationFlags::empty()));
}
