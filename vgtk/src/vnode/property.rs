use glib::Object;
use gtk::Container;

pub struct VProperty {
    pub name: &'static str,
    pub set: Box<dyn Fn(&Object, Option<&Container>, bool) + 'static>,
}
