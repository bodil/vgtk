use proc_macro2::{Group, Ident, Literal, TokenStream};
use quote::{quote, quote_spanned};

use crate::context::{Attribute, GtkComponent, GtkElement, GtkWidget};
use crate::lexer::{to_stream, Token};

fn to_string_literal<S: ToString>(s: S) -> Literal {
    Literal::string(&s.to_string())
}

fn count_attributes(attributes: &[Attribute]) -> (usize, usize, usize) {
    let mut props = 0;
    let mut child_props = 0;
    let mut handlers = 0;
    for attribute in attributes {
        match attribute {
            Attribute::Property { child, .. } => {
                if *child {
                    child_props += 1
                } else {
                    props += 1
                }
            }
            Attribute::Handler { .. } => handlers += 1,
        }
    }
    (props, child_props, handlers)
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
        use vgtk::{Component, VComponent, PropTransform};
        let mut vcomp = VComponent::new::<#name>();
        let mut props = <#name as Component>::Properties::default();
    );
    for attribute in &gtk.attributes {
        out.extend(match attribute {
            Attribute::Property {
                child,
                parent,
                name,
                value,
            } => {
                if *child {
                    let prop = expand_property(None, *child, parent, name, value);
                    quote!(
                        vcomp.child_props.push(#prop);
                    )
                } else {
                    if !parent.is_empty() {
                        panic!("component attributes cannot have paths");
                    }
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
    let name = to_stream(&gtk.name);
    let (prop_count, child_prop_count, handler_count) = count_attributes(&gtk.attributes);
    let mut out = quote!(
        use vgtk::{VNode, VHandler, VProperty, VObject, VComponent};
        use vgtk::Scope;
        use glib::StaticType;
        use std::vec::Vec;
        let object_type = #name::static_type();
        let mut properties = Vec::with_capacity(#prop_count);
        let mut child_props = Vec::with_capacity(#child_prop_count);
        let mut handlers = Vec::with_capacity(#handler_count);
        let mut children = Vec::new();
    );
    if !gtk.constructor.is_empty() {
        let cons = to_stream(&gtk.constructor);
        out.extend(quote!(
            let constructor: Option<std::boxed::Box<dyn Fn() -> glib::Object>> = Some(std::boxed::Box::new(move || {
                glib::object::Cast::upcast::<glib::Object>(#name#cons)
            }));
        ));
    } else {
        out.extend(quote!(
            let constructor = None;
        ));
    }
    for attribute in &gtk.attributes {
        out.extend(match attribute {
            Attribute::Property {
                child,
                parent,
                name,
                value,
            } => {
                let prop = expand_property(Some(&gtk.name), *child, &parent, &name, &value);
                if *child {
                    quote!(
                        child_props.push(#prop);
                    )
                } else {
                    quote!(
                        properties.push(#prop);
                    )
                }
            }
            Attribute::Handler {
                name,
                async_keyword,
                args,
                body,
            } => expand_handler(&gtk.name, &name, async_keyword.as_ref(), &args, &body),
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
        VNode::Object(VObject {
            object_type,
            constructor,
            properties,
            child_props,
            handlers,
            children,
        })
    })
}

pub fn expand_property(
    object_type: Option<&[Token]>,
    child_prop: bool,
    parent: &[Token],
    name: &Ident,
    value: &[Token],
) -> TokenStream {
    let child_prefix = if child_prop { "child_" } else { "" };
    let mut parent_type: Vec<Token> = parent.to_vec();
    while let Some(Token::Punct2(_, _, _, _)) = parent_type.last() {
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
    let value_span = value[0].span();
    let value = to_stream(value);
    let value = quote_spanned!(value_span => (#value).into_property_value());
    let prop_name = to_string_literal(name);
    let setter_prelude = if let Some(object_type) = object_type {
        let object_type = to_stream(object_type);
        quote!(
            let object: &#object_type = object.downcast_ref()
                  .unwrap_or_else(|| panic!("downcast to {:?} failed in property setter", #object_type::static_type()));
        )
    } else {
        quote!()
    };
    let setter_body = if !child_prop {
        if parent_type.is_empty() {
            quote!(
                if force || !value.compare(object.#getter()) {
                    object.#setter(value.coerce());
                }
            )
        } else {
            quote!(
                if force || !value.compare(#parent_type::#getter(object)) {
                    #parent_type::#setter(object, value.coerce());
                }
            )
        }
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
            let value = #value;
            VProperty {
                name: #prop_name,
                set: std::boxed::Box::new(move |object: &glib::Object, parent: Option<&glib::Object>, force: bool| {
                    #setter_prelude
                    #setter_body
                }),
            }
        }
    )
}

pub fn expand_handler(
    object_type: &[Token],
    name: &Ident,
    async_keyword: Option<&Token>,
    args: &[Token],
    body: &[Token],
) -> TokenStream {
    let object_type = to_stream(object_type);
    let args_s = to_stream(args);
    let body_s = to_stream(body);
    let connect = Ident::new(&format!("connect_{}", name.to_string()), name.span());
    let signal_name = to_string_literal(name);
    let location = args.first().expect("signal handler is empty!").span();
    let signal_id = to_string_literal(format!("{:?}", location));
    let inner_block = if async_keyword.is_some() {
        quote!({
            let scope = scope.clone();
            glib::MainContext::ref_thread_default().spawn_local(
                async move {
                    let msg = async move { #body_s }.await;
                    scope.send_message(msg);
                }
            )
        })
    } else {
        quote!({
            let msg = { #body_s };
            scope.send_message(msg);
        })
    };
    quote!(
        handlers.push(VHandler {
            name: #signal_name,
            id: #signal_id,
            set: std::boxed::Box::new(move |object: &glib::Object, scope: &Scope<_>| {
                let object: &#object_type = object.downcast_ref()
                      .unwrap_or_else(|| panic!("downcast to {:?} failed in signal setter", #object_type::static_type()));
                let scope: Scope<_> = scope.clone();
                object.#connect(move #args_s #inner_block)
            })
        });
    )
}
