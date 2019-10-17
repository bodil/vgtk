use gio::ApplicationFlags;
use gtk::{prelude::*, Align, Box, Button, HeaderBar, Label, Window};
use vgtk::{go, gtk, vnode::VNode, Callback, Component};

#[derive(Clone, Debug, Default)]
struct MyButton {
    label: String,
    on_clicked: Option<Callback<()>>,
}

#[derive(Clone, Debug)]
enum MyButtonMsg {
    Clicked,
}

impl Component for MyButton {
    type Message = MyButtonMsg;
    type Properties = Self;

    fn create(props: Self::Properties) -> Self {
        props
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        *self = props;
        true
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            MyButtonMsg::Clicked => {
                if let Some(ref callback) = self.on_clicked {
                    callback.send(())
                }
            }
        }
        true
    }

    fn view(&self) -> VNode<Self> {
        gtk! {
            <Button label={self.label.as_str()} on clicked=|_| {MyButtonMsg::Clicked} />
        }
    }
}

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

    fn update(&mut self, msg: Self::Message) -> bool {
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
            <Window border_width=20u32 on destroy=|_| {Message::Exit}>
                <HeaderBar title="inc!" subtitle="AD ASTRA AD INFINITVM"
                           show_close_button=true />
                <Box spacing=10 halign={Align::Center}>
                    // You can generate nodes programmaticaly by returning an
                    // IntoIterator of nodes from a block. (`VNode` itself
                    // implements IntoIterator and returns a `Once<VNode>`.)
                    { gtk! {
                        <Label label={self.counter.to_string()} />
                    } }
                    // You can insert subcomponents instead of widgets with the
                    // `@` syntax. The attributes map directly to the
                    // sub-model's `Component::Properties` struct.
                    <@MyButton label="inc!" on_clicked={|_| Message::Inc} />
                </Box>
            </Window>
        }
    }
}

fn main() {
    std::process::exit(go::<Model>("camp.lol.updog", ApplicationFlags::empty()));
}
