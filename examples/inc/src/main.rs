use vgtk::lib::gio::ApplicationFlags;
use vgtk::lib::gtk::{prelude::*, Align, Application, Box, Button, HeaderBar, Label, Window};
use vgtk::{ext::*, gtk, run, Component, UpdateAction, VNode};

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

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        match msg {
            Message::Inc => {
                self.counter += 1;
                UpdateAction::Render
            }
            Message::Exit => {
                vgtk::quit();
                UpdateAction::None
            }
        }
    }

    fn view(&self) -> VNode<Model> {
        gtk! {
            <Application::new_unwrap(Some("camp.lol.updog"), ApplicationFlags::empty())>
                <Window border_width=20 on destroy=|_| Message::Exit>
                    <HeaderBar title="inc!" subtitle="AD ASTRA AD INFINITVM"
                               show_close_button=true />
                    <Box spacing=10 halign=Align::Center>
                        <Label label=self.counter.to_string() />
                        <Button label="inc!" image="add" always_show_image=true
                                on clicked=|_| Message::Inc />
                    </Box>
                </Window>
            </Application>
        }
    }
}

fn main() {
    pretty_env_logger::init();
    std::process::exit(run::<Model>());
}
