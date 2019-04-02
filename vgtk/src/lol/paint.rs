#![recursion_limit = "4096"]

use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use gio::{ApplicationFlags, Cancellable, MemoryInputStream};
use glib::{futures::task::Context, Bytes};
use gtk::prelude::*;
use gtk::*;
use vgtk::{self, gtk, Component, VItem};

static DOG: &[u8] = include_bytes!("dog.png");

#[derive(Clone, Debug, Default)]
struct AppWindow {
    documents: Vec<Document>,
}

#[derive(Clone, Debug)]
enum AppMsg {
    Exit,
}

impl Component for AppWindow {
    type Message = AppMsg;
    type Properties = ();

    fn update(&mut self, _ctx: &mut Context, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Exit => {
                vgtk::main_quit(0);
                true
            }
        }
    }

    fn docs(&self) -> impl Iterator<Item=VItem<AppWindow>> {
        self.documents.iter().map(|doc| {
            gtk!{
                <Box name=&doc.name, title=&doc.name,>
                    { doc.view() }
                </Box>
            }
        })
    }

    fn view(&self) -> VItem<AppWindow> {
        gtk! {
            <ApplicationWindow border_width=20u32, default_width=640, default_height=480,
                    on destroy=|_| Message::Exit,>
                <HeaderBar title="GOODBOY PAINT", show_close_button=true,/>
                <Stack>
                    { self.docs() }
                </Stack>
            </ApplicationWindow>
        }
    }
}

#[derive(Clone, Debug)]
struct Document {
    name: String,
    pixbuf: Pixbuf,
}

impl Default for Document {
    fn default() -> Self {
        let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(DOG));
        let pixbuf = Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap();
        Document {
            name: "dog.png".to_string(),
            pixbuf,
        }
    }
}

#[derive(Clone, Debug)]
enum DocMsg {}

impl Component for Document {
    type Message = DocMsg;
    type Properties = ();

    fn update(&mut self, _ctx: &mut Context, msg: Self::Message) -> bool {
        false
    }

    fn view(&self) -> VItem<Document> {
        gtk! {
            <Image pixbuf=&self.pixbuf,/>
        }
    }
}

fn main() {
    let app = Application::new("camp.lol.paint", ApplicationFlags::empty()).unwrap();
    app.set_default();
    app.register(None as Option<&Cancellable>)
        .expect("application already running");
    vgtk::open::<Model>(&app);
    app.activate();
    std::process::exit(vgtk::run());
}
