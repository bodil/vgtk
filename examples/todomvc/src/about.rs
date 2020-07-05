use vgtk::lib::gdk_pixbuf::Pixbuf;
use vgtk::lib::gio::{Cancellable, MemoryInputStream};
use vgtk::lib::glib::Bytes;
use vgtk::lib::gtk::prelude::*;
use vgtk::lib::gtk::*;
use vgtk::{ext::*, gtk, run_dialog, Component, VNode};

pub struct AboutDialog {
    dog: Pixbuf,
}

static DOG: &[u8] = include_bytes!("dog.png");

impl Default for AboutDialog {
    fn default() -> Self {
        let data_stream = MemoryInputStream::from_bytes(&Bytes::from_static(DOG));
        let dog = Pixbuf::from_stream(&data_stream, None as Option<&Cancellable>).unwrap();
        AboutDialog { dog }
    }
}

impl Component for AboutDialog {
    type Message = ();
    type Properties = ();

    fn view(&self) -> VNode<Self> {
        gtk! {
            <Dialog::with_buttons(
                Some("About TodoMVC"),
                None as Option<&Window>,
                DialogFlags::MODAL,
                &[("Ok", ResponseType::Ok)]
            )>
                <Box spacing=10 orientation=Orientation::Vertical>
                    <Image pixbuf=Some(self.dog.clone())/>
                    <Label justify=Justification::Center markup="<big><b>VGTK TodoMVC</b></big>\norg-mode for dummies!"/>
                    <Label markup="<a href=\"https://github.com/bodil/vgtk\">https://github.com/bodil/vgtk</a>"/>
                </Box>
            </Dialog>
        }
    }
}

impl AboutDialog {
    #[allow(unused_must_use)]
    pub fn run() {
        run_dialog::<Self>(vgtk::current_window().as_ref());
    }
}
