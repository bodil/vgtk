use gtk::{self, Container, Widget};

use crate::component::Component;
use crate::scope::Scope;
use crate::vnode::VNode;

mod component_state;
pub use component_state::ComponentState;

mod gtk_state;
use gtk_state::GtkState;

pub enum State<Model: Component> {
    Gtk(GtkState<Model>),
    Component(ComponentState<Model>),
}

impl<Model: 'static + Component> State<Model> {
    /// Build a state from a `VItem` spec.
    pub fn build(vnode: &VNode<Model>, parent: Option<&Container>, scope: &Scope<Model>) -> Self {
        match vnode {
            VNode::Widget(widget) => State::Gtk(GtkState::build(widget, parent, scope)),
            VNode::Component(vcomp) => {
                let comp = (vcomp.constructor)(&vcomp.props, parent, &vcomp.child_props, scope);
                State::Component(comp)
            }
        }
    }

    /// Patch a state in place with a `VItem` spec.
    ///
    /// Returns true if patching succeeded, or false if a rebuild is required.
    #[must_use]
    pub fn patch(
        &mut self,
        vnode: &VNode<Model>,
        parent: Option<&Container>,
        scope: &Scope<Model>,
    ) -> bool {
        match vnode {
            VNode::Widget(widget) => match self {
                State::Gtk(state) => state.patch(widget, parent, scope),
                State::Component(_) => false,
            },
            VNode::Component(vcomp) => match self {
                State::Component(state) => state.patch(vcomp, parent, scope),
                State::Gtk(_) => false,
            },
        }
    }

    /// Get the Gtk `Widget` represented by this state.
    pub fn object(&self) -> &Widget {
        match self {
            State::Gtk(state) => &state.object,
            State::Component(state) => &state.object,
        }
    }
}
