use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        UiContainerKind, UiLayoutEngineBackend, UiLayoutEngineCapability,
        UiLayoutEngineFallbackReason, UiLayoutEngineRequest, UiLayoutEngineSelection,
        UiLayoutEngineSelectionReport, UiLayoutEngineSupport, UiLayoutEngineTaffyTreeBuildStats,
    },
};

#[derive(Debug, Default)]
pub(super) struct UiLayoutPassEngineContext {
    selections: Vec<UiLayoutEngineSelection>,
}

impl UiLayoutPassEngineContext {
    pub(super) fn record_taffy_native(
        &mut self,
        node_id: UiNodeId,
        container: UiContainerKind,
        taffy_tree_build: UiLayoutEngineTaffyTreeBuildStats,
    ) {
        self.selections.push(
            UiLayoutEngineSelection::select(
                &UiLayoutEngineRequest::from_container_kind(container),
                &UiLayoutEngineCapability::taffy_flex_grid_block(),
                &UiLayoutEngineCapability::legacy_zircon(),
            )
            .with_node_id(node_id)
            .with_taffy_tree_build(taffy_tree_build),
        );
    }

    pub(super) fn record_taffy_fallback(
        &mut self,
        node_id: UiNodeId,
        container: UiContainerKind,
        reason: UiLayoutEngineFallbackReason,
        taffy_tree_build: Option<UiLayoutEngineTaffyTreeBuildStats>,
    ) {
        self.selections.push(UiLayoutEngineSelection {
            node_id: Some(node_id),
            request: UiLayoutEngineRequest::from_container_kind(container),
            requested_backend: UiLayoutEngineBackend::Taffy,
            selected_backend: UiLayoutEngineBackend::LegacyZircon,
            support: UiLayoutEngineSupport::Fallback,
            fallback_reason: Some(reason),
            taffy_tree_build,
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
