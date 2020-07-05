use futures::channel::mpsc::UnboundedSender;
use glib::prelude::*;
use glib::{MainContext, Object};
use gtk::{prelude::*, Widget};

use std::any::TypeId;
use std::marker::PhantomData;

use crate::component::{Component, ComponentMessage, ComponentTask};
use crate::scope::Scope;
use crate::vnode::component::AnyProps;
use crate::vnode::{VComponent, VProperty};

trait PropertiesReceiver {
    fn update(&mut self, props: &AnyProps);
    fn unmounting(&self);
}

pub struct ComponentState<Model: Component> {
    parent: PhantomData<Model>,
    pub(crate) object: Object,
    model_type: TypeId,
    state: Box<dyn PropertiesReceiver>,
}

impl<Model: 'static + Component> ComponentState<Model> {
    pub fn build<Child: 'static + Component>(
        props: &AnyProps,
        parent: Option<&Object>,
        child_props: &[VProperty],
        scope: &Scope<Model>,
    ) -> Self {
        let (sub_state, object) =
            SubcomponentState::<Child>::new(props, parent, child_props, scope);
        ComponentState {
            parent: PhantomData,
            object,
            model_type: TypeId::of::<Child>(),
            state: Box::new(sub_state),
        }
    }

    pub fn patch(
        &mut self,
        spec: &VComponent<Model>,
        parent: Option<&Object>,
        _scope: &Scope<Model>,
    ) -> bool {
        if self.model_type == spec.model_type {
            // Components have same type; update props
            for prop in &spec.child_props {
                (prop.set)(self.object.upcast_ref(), parent, false);
            }
            self.state.update(&spec.props);
            true
        } else {
            // Component type changed; need to rebuild
            self.state.unmounting();
            false
        }
    }

    pub fn unmount(self) {
        self.state.unmounting();
        if let Ok(widget) = self.object.downcast::<Widget>() {
            #[allow(unsafe_code)]
            unsafe {
                widget.destroy();
            }
        }
    }
}

pub(crate) struct SubcomponentState<Model: Component> {
    channel: UnboundedSender<ComponentMessage<Model>>,
}

impl<Model: 'static + Component> SubcomponentState<Model> {
    fn new<P: 'static + Component>(
        props: &AnyProps,
        parent: Option<&Object>,
        child_props: &[VProperty],
        parent_scope: &Scope<P>,
    ) -> (Self, Object) {
        let props: Model::Properties = props.unwrap();
        let (channel, task) = ComponentTask::new(props, parent, Some(parent_scope));
        let object = task.object().unwrap();
        for prop in child_props {
            (prop.set)(object.upcast_ref(), parent, true);
        }
        MainContext::ref_thread_default().spawn_local(task);
        (SubcomponentState { channel }, object)
    }
}

impl<Model: 'static + Component> PropertiesReceiver for SubcomponentState<Model> {
    fn update(&mut self, raw_props: &AnyProps) {
        let props = raw_props.unwrap();
        self.channel
            .unbounded_send(ComponentMessage::Props(props))
            .expect("failed to send props message over system channel")
    }

    fn unmounting(&self) {
        self.channel
            .unbounded_send(ComponentMessage::Unmounted)
            .expect("failed to send unmount message over system channel")
    }
}
