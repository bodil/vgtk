use gdk_pixbuf::Pixbuf;
use gio::{Action, ActionExt, ApplicationFlags};
use glib::{GString, IsA, Object, ObjectExt};
use gtk::{
    Application, ApplicationWindowExt, BoxExt, GtkApplicationExt, GtkWindowExt, ImageExt, LabelExt,
    Window, WindowPosition, WindowType,
};

use colored::Colorize;
use log::trace;

pub trait ApplicationHelpers: GtkApplicationExt {
    fn new_unwrap(application_id: Option<&str>, flags: ApplicationFlags) -> Application {
        Application::new(application_id, flags).expect("unable to create Application object")
    }

    fn get_child_accels<P: IsA<Action>>(&self, action: &P) -> Vec<GString> {
        self.get_accels_for_action(&format!(
            "app.{}",
            action
                .as_ref()
                .get_name()
                .expect("Action has no name")
                .as_str()
        ))
    }

    fn set_child_accels<P: IsA<Action>>(&self, action: &P, accels: &[&str]) {
        self.set_accels_for_action(
            &format!(
                "app.{}",
                action
                    .as_ref()
                    .get_name()
                    .expect("Action has no name")
                    .as_str()
            ),
            accels,
        )
    }
}

impl<A> ApplicationHelpers for A where A: GtkApplicationExt {}

pub trait ApplicationWindowHelpers: ApplicationWindowExt + GtkWindowExt + IsA<Window> {
    fn get_child_accels<P: IsA<Action>>(&self, action: &P) -> Vec<GString> {
        let app = self
            .get_application()
            .expect("ApplicationWindow has no Application!");
        app.get_accels_for_action(&format!(
            "win.{}",
            action
                .as_ref()
                .get_name()
                .expect("Action has no name")
                .as_str()
        ))
    }

    fn set_child_accels<P: IsA<Action>>(&self, action: &P, accels: &'static [&str]) {
        let name = format!(
            "win.{}",
            action
                .as_ref()
                .get_name()
                .expect("Action has no name")
                .as_str()
        );
        if let Some(app) = self.get_application() {
            app.set_accels_for_action(&name, accels);
        } else {
            use std::cell::Cell;
            use std::rc::Rc;

            let id = Rc::new(Cell::new(None));
            let inner_id = id.clone();
            id.set(Some(self.connect_property_application_notify(
                move |window: &Self| {
                    if let Some(app) = window.get_application() {
                        trace!(
                            "{} {} -> {}",
                            "Action:".bright_black(),
                            name.bright_cyan().bold(),
                            format!("{:?}", accels).bright_green().bold()
                        );
                        app.set_accels_for_action(&name, accels);
                        window.disconnect(inner_id.replace(None).unwrap());
                    }
                },
            )));
        }
    }
}

impl<A> ApplicationWindowHelpers for A where A: ApplicationWindowExt + GtkWindowExt + IsA<Window> {}

pub trait WindowExtHelpers: GtkWindowExt {
    fn get_default_height(&self) -> i32 {
        self.get_property_default_height()
    }

    fn set_default_height(&self, default_height: i32) {
        self.set_property_default_height(default_height)
    }

    fn get_default_width(&self) -> i32 {
        self.get_property_default_width()
    }

    fn set_default_width(&self, default_width: i32) {
        self.set_property_default_width(default_width)
    }

    fn get_has_toplevel_focus(&self) -> bool {
        self.get_property_has_toplevel_focus()
    }

    fn get_is_active(&self) -> bool {
        self.get_property_is_active()
    }

    fn get_is_maximized(&self) -> bool {
        self.get_property_is_maximized()
    }

    fn get_type(&self) -> WindowType {
        self.get_property_type()
    }

    fn get_window_position(&self) -> WindowPosition {
        self.get_property_window_position()
    }

    fn set_window_position(&self, window_position: WindowPosition) {
        self.set_property_window_position(window_position)
    }
}

impl<A> WindowExtHelpers for A where A: GtkWindowExt {}

pub trait BoxExtHelpers: BoxExt {
    fn get_child_center_widget(&self, _child: &Object) -> bool {
        // Always compare true, it's all taken care of in add_child().
        true
    }

    fn set_child_center_widget(&self, _child: &Object, _center: bool) {
        // This is handled by add_child() rules. The setter is a no-op.
    }
}

impl<A> BoxExtHelpers for A where A: BoxExt {}

pub trait ImageExtHelpers: ImageExt {
    fn set_pixbuf(&self, pixbuf: Option<Pixbuf>) {
        self.set_from_pixbuf(pixbuf.as_ref());
    }
}

impl<A> ImageExtHelpers for A where A: ImageExt {}

pub trait LabelExtHelpers: LabelExt {
    fn get_markup(&self) -> Option<GString> {
        self.get_label()
    }
}

impl<A> LabelExtHelpers for A where A: LabelExt {}
