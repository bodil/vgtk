use proc_macro2::{Group, Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::context::{Attribute, GtkComponent, GtkElement, GtkWidget};
use crate::lexer::{to_stream, Token};

fn to_string_literal<S: ToString>(s: S) -> Literal {
    Literal::string(&s.to_string())
}

fn count_attributes(attributes: &Vec<Attribute>) -> (usize, usize) {
    let mut props = 0;
    let mut handlers = 0;
    for attribute in attributes {
        match attribute {
            Attribute::Property { .. } => props += 1,
            Attribute::Handler { .. } => handlers += 1,
        }
    }
    (props, handlers)
}

pub fn expand_gtk(gtk: &GtkElement) -> TokenStream {
    match gtk {
        GtkElement::Widget(widget) => expand_widget(widget),
        GtkElement::Component(component) => expand_component(component),
        GtkElement::Block(_block) => panic!("blocks not allowed in this position"),
    }
}

pub fn expand_component(gtk: &GtkComponent) -> TokenStream {
    let name = to_stream(&gtk.name);
    let mut out = quote!(
        use vgtk::{Component, vcomp::{VComponent, PropTransform}};
        let mut vcomp = VComponent::new::<#name>();
        let mut props = <#name as Component>::Properties::default();
    );
    for attribute in &gtk.attributes {
        out.extend(match attribute {
            Attribute::Property { name, value } => {
                let value = to_stream(value);
                quote!(
                    props.#name = PropTransform::transform(&vcomp, #value);
                )
            }
            Attribute::Handler { .. } => panic!("handler attributes are not allowed in components"),
        })
    }
    quote!({
        #out
        vcomp.set_props::<#name>(props);
        VNode::Component(vcomp)
    })
}

fn is_block(gtk: &GtkElement) -> Option<&Group> {
    match gtk {
        GtkElement::Block(block) => Some(block),
        _ => None,
    }
}

pub fn expand_widget(gtk: &GtkWidget) -> TokenStream {
    let name = &gtk.name;
    let (prop_count, handler_count) = count_attributes(&gtk.attributes);
    let mut out = quote!(
        use vgtk::vnode::{VNode, VHandler, VProperty, VWidget, VComponent, PropertyCompare};
        use vgtk::Scope;
        use glib::StaticType;
        let object_type = #name::static_type();
        let mut properties = Vec::with_capacity(#prop_count);
        let mut handlers = Vec::with_capacity(#handler_count);
        let mut children = Vec::new();
    );
    for attribute in &gtk.attributes {
        out.extend(match attribute {
            Attribute::Property { name, value } => expand_property(&gtk.name, &name, &value),
            Attribute::Handler { name, args, body } => {
                expand_handler(&gtk.name, &name, &args, &body)
            }
        });
    }
    for child in &gtk.children {
        if let Some(block) = is_block(child) {
            out.extend(quote!(
                children.extend(#block);
            ));
        } else {
            let child = expand_gtk(child);
            out.extend(quote!(
                children.push(#child);
            ));
        }
    }
    quote!({
        #out
        VNode::Widget(VWidget {
            object_type,
            properties,
            handlers,
            children,
        })
    })
}

pub fn expand_property(object_type: &Ident, name: &Ident, value: &Vec<Token>) -> TokenStream {
    let getter = Ident::new(&format!("get_{}", name.to_string()), Span::call_site());
    let setter = Ident::new(&format!("set_{}", name.to_string()), Span::call_site());
    let value = to_stream(value);
    let prop_name = to_string_literal(name);
    quote!(
        properties.push({
            let value = #value;
            VProperty {
                name: #prop_name,
                set: std::rc::Rc::new(move |object: &glib::Object, force: bool| {
                    let object: &#object_type = object.downcast_ref()
                          .unwrap_or_else(|| panic!("downcast to {:?} failed in property setter", #object_type::static_type()));
                    if force || !object.#getter().property_compare(&value) {
                        object.#setter(PropertyCompare::property_convert(&value));
                    }
                }),
            }
        });
    )
}

pub fn expand_handler(
    object_type: &Ident,
    name: &Ident,
    args: &Vec<Token>,
    body: &Vec<Token>,
) -> TokenStream {
    let args_s = to_stream(args);
    let body_s = to_stream(body);
    let connect = Ident::new(&format!("connect_{}", name.to_string()), Span::call_site());
    let signal_name = to_string_literal(name);
    let location = args.first().expect("signal handler is empty!").span();
    let signal_id = to_string_literal(format!("{:?}", location));
    quote!(
        handlers.push(VHandler {
            name: #signal_name,
            id: #signal_id,
            set: std::rc::Rc::new(|object: &glib::Object, scope: &Scope<_>| {
                let object: &#object_type = object.downcast_ref()
                      .unwrap_or_else(|| panic!("downcast to {:?} failed in signal setter", #object_type::static_type()));
                let scope: Scope<_> = scope.clone();
                object.#connect(move | #args_s | {
                    let msg = { #body_s };
                    scope.send_message(msg);
                })
            })
        });
    )
}
