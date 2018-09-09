extern crate gio;
extern crate glib;
extern crate gtk;

use glib::{Cast, IsA, Object, ObjectExt, SignalHandlerId, ToValue, Type, Value};
use gtk::{Builder, BuilderExt, Container, ContainerExt, Widget};

use std::collections::BTreeMap as OrdMap;
use std::rc::Rc;

pub struct SignalConnector {
    pub connect: Box<Fn(&Object) -> SignalHandlerId>,
}

pub struct VObject {
    pub type_: Type,
    properties: OrdMap<String, Value>,
    handlers: Vec<SignalConnector>,
    children: Vec<Rc<VObject>>,
}

fn build_obj(class: Type, id: Option<&str>) -> Object {
    let mut ui = String::new();
    ui += &format!("<interface><object class=\"{}\"", class);
    if let Some(id) = id {
        ui += &format!(" id=\"{}\"", id);
    }
    ui += "/></interface>";

    let builder = Builder::new_from_string(&ui);
    let objects = builder.get_objects();
    objects
        .last()
        .unwrap_or_else(|| panic!("unknown class {}", class))
        .clone()
}

impl VObject {
    pub fn new(type_: Type) -> Self {
        VObject {
            type_,
            properties: Default::default(),
            handlers: Default::default(),
            children: Vec::new(),
        }
    }

    pub fn set_property<Prop, Val>(&mut self, prop: Prop, value: &Val)
    where
        Prop: Into<String>,
        Val: ToValue,
    {
        self.properties.insert(prop.into(), value.to_value());
    }

    pub fn add_handler(&mut self, connector: SignalConnector) {
        self.handlers.push(connector)
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(Rc::new(child))
    }

    fn construct(&self) -> Object {
        let mut obj = build_obj(
            self.type_,
            self.properties
                .get("id")
                .map(|v| v.get().expect("id property is not a string!")),
        );

        for (prop, value) in &self.properties {
            if prop != "id" {
                obj.set_property(prop.as_str(), value)
                    .unwrap_or_else(|_| panic!("Class {} has no property {:?}", self.type_, prop));
            }
        }

        for connector in &self.handlers {
            (connector.connect)(&obj);
        }

        if !self.children.is_empty() {
            let parent: Container = obj.downcast().expect("non-Container parent");
            for child_spec in &self.children {
                let child: Widget = child_spec.construct().downcast().expect("non-Widget child");
                parent.add(&child);
            }

            obj = parent.upcast();
        }

        obj
    }

    pub fn build<A>(&self) -> A
    where
        A: IsA<Object>,
    {
        self.construct().downcast().unwrap()
    }
}

#[macro_export]
macro_rules! gtk {
    ( $stack:ident (< $class:ident $($tail:tt)*)) => {
        {
            let obj = $crate::VObject::new($class::static_type());
            $stack.push(obj);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident ( on $signal:ident = |$args:pat| $handler:expr, $($tail:tt)* )) => {
        {
            let obj = $stack.last_mut().expect("stack was empty!");
            let connector = $crate::SignalConnector {
                connect: Box::new(|o| o.clone().downcast::<$class>().unwrap().$signal(move |$args| $handler))
            };
            obj.add_handler(connector);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident ( on $signal:ident = $handler:expr, $($tail:tt)* )) => {
        {
            let obj = $stack.last_mut().expect("stack was empty!");
            let connector = $crate::SignalConnector {
                connect: Box::new(|o| o.clone().downcast::<$class>().unwrap().$signal($handler))
            };
            obj.add_handler(connector);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident ( $prop:ident = $value:expr, $($tail:tt)* )) => {
        {
            let obj = $stack.last_mut().expect("stack was empty!");
            obj.set_property(stringify!($prop), &$value);
        }
        gtk!{ @obj $class $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident (/ > $($tail:tt)*)) => {
        {
            let child = $stack.pop().unwrap();
            if !$stack.is_empty() {
                let parent = $stack.last_mut().unwrap();
                parent.add_child(child);
            } else {
                $stack.push(child);
            }
        }
        gtk!{ $stack ($($tail)*) }
    };
    (@obj $class:ident $stack:ident (> $($tail:tt)*)) => {
        gtk!{ $stack ($($tail)*) }
    };
    ( $stack:ident (< / $class:ident > $($tail:tt)*)) => {
        {
            let child = $stack.pop().unwrap();
            debug_assert!(child.type_ == $class::static_type());
            if !$stack.is_empty() {
                let parent = $stack.last_mut().unwrap();
                parent.add_child(child);
            } else {
                $stack.push(child);
            }
        }
        gtk!{ $stack ($($tail)*) }
    };
    ($stack:ident ()) => {
        $stack.pop().expect("empty gtk! macro")
    };
    ($($tail:tt)*) => {{
        let mut stack: Vec<$crate::VObject> = Vec::new();
        gtk!{ stack ($($tail)*) }
    }}
}
