use crate::Component;

pub(crate) mod component;
mod gobject;
mod handler;
mod property;

pub use component::{PropTransform, VComponent};
pub use gobject::VObject;
pub use handler::VHandler;
pub use property::VProperty;

/// A node in the virtual component tree representing a [`Component`][Component] or a Gtk widget.
///
/// Don't attempt to construct these directly. Use the [`gtk!`][gtk!] macro instead.
///
/// [gtk!]: macro.gtk.html
/// [Component]: trait.Component.html
pub enum VNode<Model: Component> {
    Object(VObject<Model>),
    Component(VComponent<Model>),
}

impl<Model: Component> VNode<Model> {
    pub(crate) fn get_child_props(&self) -> &[VProperty] {
        match self {
            VNode::Object(object) => &object.child_props,
            VNode::Component(comp) => &comp.child_props,
        }
    }

    pub(crate) fn get_child_prop(&self, name: &str) -> Option<&VProperty> {
        let props = self.get_child_props();
        for prop in props {
            if prop.name == name {
                return Some(prop);
            }
        }
        None
    }
}

/// An iterator over zero or one [`VNode`][VNode]s.
///
/// A [`VNode`][VNode] implements [`IntoIterator`][IntoIterator] to build a `VNodeIterator`, so
/// you can return a single [`VNode`][VNode] in a code block in the [`gtk!`][gtk!] macro without
/// needing to convert it.
///
/// If you need to return an empty list of [`VNode`][VNode]s, use [`VNode::empty()`][empty].
///
/// This iterator mainly exists to enable the pattern of conditionally returning a node from an
/// if expression, because the empty iterator and the single node iterator both have the same type,
/// unlike if you attempted to return [`std::iter::once()`][iter::once] and
/// [`std::iter::empty()`][iter::empty], which would result in a type error.
///
/// # Examples
///
/// ```rust,no_run
/// # use vgtk::{gtk, VNode};
/// # use vgtk::lib::gtk::{Label, LabelExt, Box, BoxExt, Orientation};
/// # fn should_render_label() -> bool { true }
/// # fn view() -> VNode<()> {
/// let render_label: bool = should_render_label();
/// gtk! {
///     <Box::new(Orientation::Vertical, 10)>
///         {
///             if render_label {
///                 (gtk! { <Label label="It's a me, Label!" /> }).into_iter()
///             } else {
///                 VNode::empty()
///             }
///         }
///     </Box>
/// }
/// # }
/// ```
///
/// [gtk!]: macro.gtk.html
/// [VNode]: enum.VNode.html
/// [empty]: enum.VNode.html#method.empty
/// [IntoIterator]: https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
/// [iter::once]: https://doc.rust-lang.org/std/iter/fn.once.html
/// [iter::empty]: https://doc.rust-lang.org/std/iter/fn.empty.html
pub struct VNodeIterator<Model: Component> {
    node: Option<VNode<Model>>,
}

impl<Model: Component> Iterator for VNodeIterator<Model> {
    type Item = VNode<Model>;
    fn next(&mut self) -> Option<Self::Item> {
        self.node.take()
    }
}

impl<Model: Component> IntoIterator for VNode<Model> {
    type Item = VNode<Model>;
    type IntoIter = VNodeIterator<Model>;
    fn into_iter(self) -> Self::IntoIter {
        VNodeIterator { node: Some(self) }
    }
}

impl<Model: Component> VNode<Model> {
    /// Make an empty iterator of [`VNode`][VNode]s.
    ///
    /// Use this inside a code block in the [`gtk!`][gtk!] macro to return an empty list
    /// of child nodes.
    ///
    /// [gtk!]: macro.gtk.html
    /// [VNode]: enum.VNode.html
    pub fn empty() -> VNodeIterator<Model> {
        VNodeIterator { node: None }
    }
}
