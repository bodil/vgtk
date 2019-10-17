use gtk::{GtkWindowExt, Window, WindowPosition, WindowType};

pub trait WindowExtHelpers: GtkWindowExt {
    fn get_default_height(&self) -> i32;
    fn set_default_height(&self, default_height: i32);
    fn get_default_width(&self) -> i32;
    fn set_default_width(&self, default_width: i32);
    fn get_has_toplevel_focus(&self) -> bool;
    fn get_is_active(&self) -> bool;
    fn get_is_maximized(&self) -> bool;
    fn get_type(&self) -> WindowType;
    fn get_window_position(&self) -> WindowPosition;
    fn set_window_position(&self, window_position: WindowPosition);
}

impl WindowExtHelpers for Window {
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
