use proc_macro2::{Group, Ident, Literal, TokenStream};
use quote::quote;

use crate::context::{Attribute, GtkComponent, GtkElement, GtkWidget};
use crate::lexer::{to_stream, Token};

fn to_string_literal<S: ToString>(s: S) -> Literal {
    Literal::string(&s.to_string())
}

fn count_attributes(attributes: &[Attribute]) -> (usize, usize) {
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
            Attribute::Property {
                parent,
                name,
                value,
            } => {
                if !parent.is_empty() {
                    let prop = expand_property(None, parent, name, value);
                    quote!(
                        vcomp.child_props.push(#prop);
                    )
                } else {
                    let value = to_stream(value);
                    quote!(
                        props.#name = PropTransform::transform(&vcomp, #value);
                    )
                }
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
        use vgtk::vnode::{VNode, VHandler, VProperty, VWidget, VComponent};
        use vgtk::Scope;
        use glib::StaticType;
        let object_type = #name::static_type();
        let mut properties = Vec::with_capacity(#prop_count);
        let mut handlers = Vec::with_capacity(#handler_count);
        let mut children = Vec::new();
    );
    for attribute in &gtk.attributes {
        out.extend(match attribute {
            Attribute::Property {
                parent,
                name,
                value,
            } => {
                let prop = expand_property(Some(&gtk.name), &parent, &name, &value);
                quote!(
                    properties.push(#prop);
                )
            }
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

pub fn expand_property(
    object_type: Option<&Ident>,
    parent: &[Token],
    name: &Ident,
    value: &[Token],
) -> TokenStream {
    let child_prefix = if !parent.is_empty() { "child_" } else { "" };
    let mut parent_type: Vec<Token> = parent.to_vec();
    while let Some(Token::Punct(_, _)) = parent_type.last() {
        parent_type.pop();
    }
    let parent_type = to_stream(parent_type.iter());
    let getter = Ident::new(
        &format!("get_{}{}", child_prefix, name.to_string()),
        name.span(),
    );
    let setter = Ident::new(
        &format!("set_{}{}", child_prefix, name.to_string()),
        name.span(),
    );
    let value = to_stream(value);
    let prop_name = to_string_literal(name);
    let setter_prelude = if let Some(object_type) = object_type {
        quote!(
            let object: &#object_type = object.downcast_ref()
                  .unwrap_or_else(|| panic!("downcast to {:?} failed in property setter", #object_type::static_type()));
        )
    } else {
        quote!(
            let object: &Widget = object.downcast_ref()
                  .expect("downcast to Widget failed in property setter");
        )
    };
    let setter_body = if parent.is_empty() {
        quote!(
            if force || !value.compare(object.#getter()) {
                object.#setter(value.coerce());
            }
        )
    } else {
        quote!(
            let parent: &#parent_type = parent.expect("child attribute without a reachable parent").downcast_ref()
                  .unwrap_or_else(|| panic!("downcast to {:?} failed on parent in property setter", #parent_type::static_type()));
            if force || !value.compare(parent.#getter(object)) {
                parent.#setter(object, value.coerce());
            }
        )
    };
    quote!(
        {
            use gtk::{Container, Widget};
            use glib::StaticType;
            use vgtk::properties::{
                IntoPropertyValue, PropertyValue, PropertyValueCoerce, PropertyValueCompare,
            };
            let value = #value.into_property_value();
            VProperty {
                name: #prop_name,
                set: std::rc::Rc::new(move |object: &glib::Object, parent: Option<&Container>, force: bool| {
                    #setter_prelude
                    #setter_body
                }),
            }
        }
    )
}

pub fn expand_handler(
    object_type: &Ident,
    name: &Ident,
    args: &[Token],
    body: &[Token],
) -> TokenStream {
    let args_s = to_stream(args);
    let body_s = to_stream(body);
    let connect = Ident::new(&format!("connect_{}", name.to_string()), name.span());
    let signal_name = to_string_literal(name);
    let location = args.first().expect("signal handler is empty!").span();
    let signal_id = to_string_literal(format!("{:?}", location));
    quote!(
        handlers.push(VHandler {
            name: #signal_name,
            id: #signal_id,
            set: std::rc::Rc::new(move |object: &glib::Object, scope: &Scope<_>| {
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
