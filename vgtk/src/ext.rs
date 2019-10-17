use glib::object::Cast;
use gtk::{Box, BoxExt, GtkWindowExt, IsA, PackType, Widget, Window, WindowPosition, WindowType};

use crate::ffi;

// pub trait BoxExtHelpers: BoxExt {
//     fn get_child_expand<P: IsA<Widget>>(&self, child: &P) -> bool;
//     fn set_child_expand<P: IsA<Widget>>(&self, child: &P, expand: bool);
//     fn get_child_fill<P: IsA<Widget>>(&self, child: &P) -> bool;
//     fn set_child_fill<P: IsA<Widget>>(&self, child: &P, fill: bool);
//     fn get_child_padding<P: IsA<Widget>>(&self, child: &P) -> u32;
//     fn set_child_padding<P: IsA<Widget>>(&self, child: &P, padding: u32);
//     fn get_child_pack_type<P: IsA<Widget>>(&self, child: &P) -> PackType;
//     fn set_child_pack_type<P: IsA<Widget>>(&self, child: &P, pack_type: PackType);
// }
//
// impl BoxExtHelpers for Box {
//     fn get_child_expand<P: IsA<Widget>>(&self, child: &P) -> bool {
//         self.query_child_packing(child).0
//     }
//
//     fn set_child_expand<P: IsA<Widget>>(&self, child: &P, expand: bool) {
//         ffi::set_child_property(self.upcast_ref(), child.as_ref(), "expand", &expand)
//             .unwrap_or_else(|err| panic!("failed to set child property: {:?}", err));
//     }
//
//     fn get_child_fill<P: IsA<Widget>>(&self, child: &P) -> bool {
//         self.query_child_packing(child).1
//     }
//
//     fn set_child_fill<P: IsA<Widget>>(&self, child: &P, fill: bool) {
//         ffi::set_child_property(self.upcast_ref(), child.as_ref(), "fill", &fill)
//             .unwrap_or_else(|err| panic!("failed to set child property: {:?}", err));
//     }
//
//     fn get_child_padding<P: IsA<Widget>>(&self, child: &P) -> u32 {
//         self.query_child_packing(child).2
//     }
//
//     fn set_child_padding<P: IsA<Widget>>(&self, child: &P, padding: u32) {
//         ffi::set_child_property(self.upcast_ref(), child.as_ref(), "padding", &padding)
//             .unwrap_or_else(|err| panic!("failed to set child property: {:?}", err));
//     }
//
//     fn get_child_pack_type<P: IsA<Widget>>(&self, child: &P) -> PackType {
//         self.query_child_packing(child).3
//     }
//
//     fn set_child_pack_type<P: IsA<Widget>>(&self, child: &P, pack_type: PackType) {
//         ffi::set_child_property(self.upcast_ref(), child.as_ref(), "pack_type", &pack_type)
//             .unwrap_or_else(|err| panic!("failed to set child property: {:?}", err));
//     }
// }

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
