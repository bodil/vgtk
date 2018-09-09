extern crate gio;
extern crate glib;
extern crate gtk;

#[macro_export]
macro_rules! gtk {
    ($stack:ident (< $class:ident($($arg:expr),*) $($tail:tt)*)) => {{
        let obj = $class::new($($arg,)*);
        $stack.push(obj.to_value());
        gtk!{ @obj $class $stack ($($tail)*) }
    }};
    ($stack:ident (< $class:ident $($tail:tt)*)) => {{
        let obj = $class::new();
        $stack.push(obj.to_value());
        gtk!{ @obj $class $stack ($($tail)*) }
    }};
    (@obj $class:ident $stack:ident ( on $signal:ident = |$args:pat| $handler:expr, $($tail:tt)* )) => {{
        let obj: $class = $stack.last().unwrap().get()
            .unwrap_or_else(|| panic!("type mismatch in gtk! macro: expected {}, got {}",
                                      stringify!($class), $stack.last().unwrap().type_()));
        obj.$signal(move |$args| $handler);
        gtk!{ @obj $class $stack ($($tail)*) }
    }};
    (@obj $class:ident $stack:ident ( on $signal:ident = $handler:expr, $($tail:tt)* )) => {{
        let obj: $class = $stack.last().unwrap().get()
            .unwrap_or_else(|| panic!("type mismatch in gtk! macro: expected {}, got {}",
                                      stringify!($class), $stack.last().unwrap().type_()));
        obj.$signal($handler);
        gtk!{ @obj $class $stack ($($tail)*) }
    }};
    (@obj $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {{
        let obj: $class = $stack.last().expect("stack was empty!").get().expect("not an Object");
        obj.set_property(stringify!($prop), &$value).expect("failed to set property");
        gtk!{ @obj $class $stack ($($tail)*) }
    }};
    (@obj $class:ident $stack:ident (/ > $($tail:tt)*)) => {
        let child_value = $stack.pop().unwrap();
        let child: $class = child_value.get()
            .unwrap_or_else(|| panic!("type mismatch in gtk! macro: expected {}, got {}",
                                      stringify!($class), child_value.type_()));
        if !$stack.is_empty() {
            let parent_value = $stack.last().unwrap();
            let parent: ::gtk::Container = parent_value.get()
                .unwrap_or_else(|| panic!("in gtk! macro: {} is not a Container", parent_value.type_()));
            parent.add(&child);
        } else {
            $stack.push(child_value);
        }
        gtk!{ $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident (> $($tail:tt)*)) => {
        gtk!{ $stack ($($tail)*) }
    };
    ($stack:ident (< / $class:ident > $($tail:tt)*)) => {
        let child_value = $stack.pop().expect("in gtk! macro: closing tag without opening tag!");
        debug_assert!(child_value.is::<$class>());
        if !$stack.is_empty() {
            let parent_value = $stack.last().unwrap();
            let parent: ::gtk::Container = parent_value.get()
                .unwrap_or_else(|| panic!("in gtk! macro: {} is not a Container", parent_value.type_()));
            let child: $class = child_value.get()
                .unwrap_or_else(|| panic!("type mismatch in gtk! macro: expected {}, got {}",
                                          stringify!($class), child_value.type_()));
            parent.add(&child);
        } else {
            $stack.push(child_value);
        }
        gtk!{ $stack ($($tail)*) }
    };
    ($stack:ident ()) => {
        let result = $stack.pop().expect("empty gtk! macro");
        result.get().unwrap_or_else(|| panic!("in gtk! macro: cannot cast toplevel {:?} object to return type", result.type_()))
    };
    ($($tail:tt)*) => {{
        let mut stack: Vec<::glib::Value> = Vec::new();
        gtk!{ stack ($($tail)*) }
    }}
}
