use glib::Object;

pub struct VProperty {
    pub name: &'static str,
    pub set: Box<dyn Fn(&Object, Option<&Object>, bool) + 'static>,
}
