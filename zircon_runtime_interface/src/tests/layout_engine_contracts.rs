use crate::ui::layout::{
    UiContainerKind, UiLayoutEngineBackend, UiLayoutEngineCapability, UiLayoutEngineFallbackReason,
    UiLayoutEngineFamily, UiLayoutEngineRequest, UiLayoutEngineSelection,
    UiLayoutEngineSelectionReport, UiLayoutEngineSupport, UiLinearBoxConfig, UiScrollableBoxConfig,
    UiVirtualListConfig,
};

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn ui_layout_engine_capability_distinguishes_taffy_compatible_and_zircon_owned_families() {
    let capability = UiLayoutEngineCapability::taffy_flex_grid_block();

    assert!(capability.supports_family(UiLayoutEngineFamily::Flex));
    assert!(capability.supports_family(UiLayoutEngineFamily::Grid));
    assert!(capability.supports_family(UiLayoutEngineFamily::Block));
    assert!(!capability.supports_family(UiLayoutEngineFamily::Overlay));
    assert!(!capability.supports_family(UiLayoutEngineFamily::Scrollable));
    assert!(!capability.supports_family(UiLayoutEngineFamily::VirtualizedList));
    assert!(capability.supports_content_measure);
    assert_eq!(round_trip(&capability), capability);
}

#[test]
fn ui_layout_engine_request_maps_current_container_contracts_to_engine_families() {
    let horizontal = UiLayoutEngineRequest::from_container_kind(UiContainerKind::HorizontalBox(
        UiLinearBoxConfig { gap: 8.0 },
    ));
    let grid =
        UiLayoutEngineRequest::from_container_kind(UiContainerKind::GridBox(Default::default()));
    let overlay = UiLayoutEngineRequest::from_container_kind(UiContainerKind::Overlay);
    let scrollable = UiLayoutEngineRequest::from_container_kind(UiContainerKind::ScrollableBox(
        UiScrollableBoxConfig {
            virtualization: Some(UiVirtualListConfig {
                item_extent: 24.0,
                overscan: 2,
            }),
            ..UiScrollableBoxConfig::default()
        },
    ));

    assert_eq!(horizontal.family, UiLayoutEngineFamily::Flex);
    assert_eq!(grid.family, UiLayoutEngineFamily::Grid);
    assert_eq!(overlay.family, UiLayoutEngineFamily::Overlay);
    assert_eq!(scrollable.family, UiLayoutEngineFamily::VirtualizedList);
    assert!(scrollable.requires_zircon_semantics());
    assert_eq!(round_trip(&scrollable), scrollable);
}

#[test]
fn ui_layout_engine_selection_reports_backend_fallbacks_without_running_layout() {
    let taffy = UiLayoutEngineCapability::taffy_flex_grid_block();
    let legacy = UiLayoutEngineCapability::legacy_zircon();
    let flex = UiLayoutEngineRequest::new(UiLayoutEngineFamily::Flex);
    let overlay = UiLayoutEngineRequest::new(UiLayoutEngineFamily::Overlay);
    let scrollable = UiLayoutEngineRequest::new(UiLayoutEngineFamily::Scrollable);

    let taffy_flex = UiLayoutEngineSelection::select(&flex, &taffy, &legacy);
    let taffy_overlay = UiLayoutEngineSelection::select(&overlay, &taffy, &legacy);
    let taffy_scrollable = UiLayoutEngineSelection::select(&scrollable, &taffy, &legacy);
    let report = UiLayoutEngineSelectionReport::from_selections(vec![
        taffy_flex.clone(),
        taffy_overlay.clone(),
        taffy_scrollable.clone(),
    ]);

    assert_eq!(taffy_flex.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(taffy_flex.support, UiLayoutEngineSupport::Native);
    assert_eq!(
        taffy_overlay.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(
        taffy_overlay.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
    assert_eq!(
        taffy_scrollable.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(report.request_count, 3);
    assert_eq!(report.taffy_selected_count, 1);
    assert_eq!(report.legacy_selected_count, 2);
    assert_eq!(report.fallback_count, 2);
    assert_eq!(round_trip(&report), report);
}
