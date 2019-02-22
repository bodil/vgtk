use gobject_sys;
use gtk_sys;

use glib::translate::{from_glib, from_glib_none, mut_override, ToGlib, ToGlibPtr};
use glib::{glib_bool_error, BoolError, ParamFlags, ParamSpec, ToValue};
use gtk::{Container, Widget};

use std::os::raw::c_char;

extern "C" {
    fn gtk_container_class_find_child_property(
        klass: *mut gtk_sys::GtkContainerClass,
        prop: *const c_char,
    ) -> *mut gobject_sys::GParamSpec;
}

pub fn find_child_property<'a, P: Into<&'a str>>(parent: &Container, prop: P) -> Option<ParamSpec> {
    let prop = prop.into();
    unsafe {
        let obj: *const gtk_sys::GtkContainer = parent.to_glib_none().0;
        let klass = (*(obj as *const gobject_sys::GObject))
            .g_type_instance
            .g_class as *mut gobject_sys::GObjectClass;

        from_glib_none(gtk_container_class_find_child_property(
            klass as *mut gtk_sys::GtkContainerClass,
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
        None => return Err(glib_bool_error!("property not found")),
    };

    if !pspec.get_flags().contains(ParamFlags::WRITABLE)
        || pspec.get_flags().contains(ParamFlags::CONSTRUCT_ONLY)
    {
        return Err(glib_bool_error!("property is not writable"));
    }

    unsafe {
        let valid_type: bool = from_glib(gobject_sys::g_type_check_value_holds(
            value.to_glib_none().0,
            pspec.get_value_type().to_glib(),
        ));
        if !valid_type {
            return Err(glib_bool_error!(
                "property can't be set from the given type"
            ));
        }

        let changed: bool = from_glib(gobject_sys::g_param_value_validate(
            pspec.to_glib_none().0,
            mut_override(value.to_glib_none().0),
        ));
        let change_allowed = pspec.get_flags().contains(ParamFlags::LAX_VALIDATION);
        if changed && !change_allowed {
            return Err(glib_bool_error!(
                "property can't be set from given value, it is invalid or out of range",
            ));
        }

        gtk_sys::gtk_container_child_set_property(
            parent.to_glib_none().0,
            child.to_glib_none().0,
            prop.to_glib_none().0,
            value.to_glib_none().0,
        )
    }

    Ok(())
}
