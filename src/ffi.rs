use gobject_ffi;
use gtk_ffi;

use glib::translate::{from_glib, from_glib_none, mut_override, ToGlib, ToGlibPtr};
use glib::{BoolError, ParamFlags, ParamSpec, ToValue};
use gtk::{Container, Widget};

use std::os::raw::c_char;

extern "C" {
    fn gtk_container_class_find_child_property(
        klass: *mut gtk_ffi::GtkContainerClass,
        prop: *const c_char,
    ) -> *mut gobject_ffi::GParamSpec;
}

pub fn find_child_property<'a, P: Into<&'a str>>(parent: &Container, prop: P) -> Option<ParamSpec> {
    let prop = prop.into();
    unsafe {
        let obj: *const gtk_ffi::GtkContainer = parent.to_glib_none().0;
        let klass = (*(obj as *const gobject_ffi::GObject))
            .g_type_instance
            .g_class as *mut gobject_ffi::GObjectClass;

        from_glib_none(gtk_container_class_find_child_property(
            klass as *mut gtk_ffi::GtkContainerClass,
            prop.to_glib_none().0,
        ))
    }
}

pub fn set_child_property<'a, P: Into<&'a str>>(
    parent: &Container,
    child: &Widget,
    prop: P,
    value: &ToValue,
) -> Result<(), BoolError> {
    let prop = prop.into();
    let value = value.to_value();

    let pspec = match find_child_property(parent, prop) {
        Some(pspec) => pspec,
        None => return Err(BoolError("property not found")),
    };

    if !pspec.get_flags().contains(ParamFlags::WRITABLE)
        || pspec.get_flags().contains(ParamFlags::CONSTRUCT_ONLY)
    {
        return Err(BoolError("property is not writable"));
    }

    unsafe {
        let valid_type: bool = from_glib(gobject_ffi::g_type_check_value_holds(
            value.to_glib_none().0,
            pspec.get_value_type().to_glib(),
        ));
        if !valid_type {
            return Err(BoolError("property can't be set from the given type"));
        }

        let changed: bool = from_glib(gobject_ffi::g_param_value_validate(
            pspec.to_glib_none().0,
            mut_override(value.to_glib_none().0),
        ));
        let change_allowed = pspec.get_flags().contains(ParamFlags::LAX_VALIDATION);
        if changed && !change_allowed {
            return Err(BoolError(
                "property can't be set from given value, it is invalid or out of range",
            ));
        }

        gtk_ffi::gtk_container_child_set_property(
            parent.to_glib_none().0,
            child.to_glib_none().0,
            prop.to_glib_none().0,
            value.to_glib_none().0,
        )
    }

    Ok(())
}
