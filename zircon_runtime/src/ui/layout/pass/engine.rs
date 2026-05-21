use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        UiContainerKind, UiLayoutEngineBackend, UiLayoutEngineCapability,
        UiLayoutEngineFallbackReason, UiLayoutEngineRequest, UiLayoutEngineSelection,
        UiLayoutEngineSelectionReport, UiLayoutEngineSupport,
    },
};

#[derive(Debug, Default)]
pub(super) struct UiLayoutPassEngineContext {
    selections: Vec<UiLayoutEngineSelection>,
}

impl UiLayoutPassEngineContext {
    pub(super) fn record_taffy_native(&mut self, node_id: UiNodeId, container: UiContainerKind) {
        self.selections.push(
            UiLayoutEngineSelection::select(
                &UiLayoutEngineRequest::from_container_kind(container),
                &UiLayoutEngineCapability::taffy_flex_grid_block(),
                &UiLayoutEngineCapability::legacy_zircon(),
            )
            .with_node_id(node_id),
        );
    }

    pub(super) fn record_taffy_fallback(
        &mut self,
        node_id: UiNodeId,
        container: UiContainerKind,
        reason: UiLayoutEngineFallbackReason,
    ) {
        self.selections.push(UiLayoutEngineSelection {
            node_id: Some(node_id),
            request: UiLayoutEngineRequest::from_container_kind(container),
            requested_backend: UiLayoutEngineBackend::Taffy,
            selected_backend: UiLayoutEngineBackend::LegacyZircon,
            support: UiLayoutEngineSupport::Fallback,
            fallback_reason: Some(reason),
        });
    }

    pub(super) fn record_zircon_owned(&mut self, node_id: UiNodeId, container: UiContainerKind) {
        self.selections.push(
            UiLayoutEngineSelection::select(
                &UiLayoutEngineRequest::from_container_kind(container),
                &UiLayoutEngineCapability::taffy_flex_grid_block(),
                &UiLayoutEngineCapability::legacy_zircon(),
            )
            .with_node_id(node_id),
        );
    }

    pub(super) fn finish(self) -> UiLayoutEngineSelectionReport {
        UiLayoutEngineSelectionReport::from_selections(self.selections)
    }
}
