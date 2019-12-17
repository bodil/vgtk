use glib::{Cast, Object};
use gtk::{self, Widget};

use crate::component::Component;
use crate::scope::Scope;
use crate::vnode::VNode;

mod component_state;
pub(crate) use component_state::ComponentState;

mod gtk_state;
use gtk_state::GtkState;

pub(crate) enum State<Model: Component> {
    Gtk(GtkState<Model>),
    Component(ComponentState<Model>),
}

impl<Model: 'static + Component> State<Model> {
    /// Build a full state from a `VItem` spec.
    pub(crate) fn build(
        vnode: &VNode<Model>,
        parent: Option<&Object>,
        scope: &Scope<Model>,
    ) -> Self {
        match vnode {
            VNode::Object(object) => State::Gtk(GtkState::build(object, parent, scope)),
            VNode::Component(vcomp) => {
                let comp = (vcomp.constructor)(&vcomp.props, parent, &vcomp.child_props, scope);
                State::Component(comp)
            }
        }
    }

    /// Build a full state from a `VItem` spec.
    pub(crate) fn build_root(
        vnode: &VNode<Model>,
        parent: Option<&Object>,
        scope: &Scope<Model>,
    ) -> Self {
        match vnode {
            VNode::Object(object) => State::Gtk(GtkState::build_root(object, parent, scope)),
            VNode::Component(_vcomp) => {
                // let comp = (vcomp.constructor)(&vcomp.props, parent, &vcomp.child_props, scope);
                // State::Component(comp)
                unimplemented!()
            }
        }
    }

    pub(crate) fn build_children(&mut self, vnode: &VNode<Model>, scope: &Scope<Model>) {
        match vnode {
            VNode::Object(vobject) => match self {
                State::Gtk(gtk_state) => gtk_state.build_children(vobject, scope),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    /// Patch a state in place with a `VItem` spec.
    ///
    /// Returns true if patching succeeded, or false if a rebuild is required.
    #[must_use]
    pub(crate) fn patch(
        &mut self,
        vnode: &VNode<Model>,
        parent: Option<&Object>,
        scope: &Scope<Model>,
    ) -> bool {
        match vnode {
            VNode::Object(object) => match self {
                State::Gtk(state) => state.patch(object, parent, scope),
                State::Component(_) => false,
            },
            VNode::Component(vcomp) => match self {
                State::Component(state) => state.patch(vcomp, parent, scope),
                State::Gtk(_) => false,
            },
        }
    }

    pub(crate) fn unmount(self) {
        match self {
            State::Gtk(state) => state.unmount(),
            State::Component(state) => state.unmount(),
        }
    }

    /// Get the Glib `Object` represented by this state.
    pub(crate) fn object(&self) -> &Object {
        match self {
            State::Gtk(state) => &state.object,
            State::Component(state) => &state.object,
        }
    }

    /// Get the Gtk `Widget` represented by this state, if it has a `Widget`.
    pub(crate) fn widget(&self) -> Option<&Widget> {
        match self {
            State::Gtk(state) => state.object.downcast_ref::<Widget>(),
            State::Component(state) => state.object.downcast_ref::<Widget>(),
        }
    }
}
