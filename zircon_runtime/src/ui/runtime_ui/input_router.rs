use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchPhase, UiNavigationDispatchContext, UiNavigationDispatchEffect,
        UiPointerDispatchContext, UiPointerDispatchEffect,
    },
    event_ui::UiNodeId,
    surface::{UiNavigationEventKind, UiPointerEventKind},
};

/// Runtime-owned input router matching SlateApplication-style ownership:
/// platform/runtime events enter the manager, then one retained dispatcher set
/// routes pointer and navigation replies through the shared UiSurface.
#[derive(Default)]
pub(super) struct RuntimeUiInputRouter {
    pointer: UiPointerDispatcher,
    navigation: UiNavigationDispatcher,
}

impl RuntimeUiInputRouter {
    pub(super) fn clear_node_handlers(&mut self) {
        *self = Self::default();
    }

    pub(super) fn pointer(&self) -> &UiPointerDispatcher {
        &self.pointer
    }

    pub(super) fn navigation(&self) -> &UiNavigationDispatcher {
        &self.navigation
    }

    pub(super) fn register_pointer<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        handler: F,
    ) where
        F: Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static,
    {
        self.pointer.register(node_id, kind, handler);
    }

    pub(super) fn register_pointer_phase<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        phase: UiDispatchPhase,
        handler: F,
    ) where
        F: Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static,
    {
        self.pointer.register_phase(node_id, kind, phase, handler);
    }

    pub(super) fn register_navigation<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiNavigationEventKind,
        handler: F,
    ) where
        F: Fn(&UiNavigationDispatchContext) -> UiNavigationDispatchEffect + Send + Sync + 'static,
    {
        self.navigation.register(node_id, kind, handler);
    }
}
