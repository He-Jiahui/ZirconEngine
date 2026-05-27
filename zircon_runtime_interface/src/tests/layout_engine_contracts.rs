use crate::ui::layout::{
    UiContainerKind, UiLayoutEngineBackend, UiLayoutEngineCapability, UiLayoutEngineFallbackReason,
    UiLayoutEngineFamily, UiLayoutEngineRequest, UiLayoutEngineSelection,
    UiLayoutEngineSelectionReport, UiLayoutEngineSupport, UiLayoutEngineTaffyTreeBuildStats,
    UiLinearBoxConfig, UiMasonryBoxConfig, UiScrollableBoxConfig, UiSizeBoxConfig,
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
    assert!(!capability.supports_family(UiLayoutEngineFamily::Masonry));
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
    let masonry = UiLayoutEngineRequest::from_container_kind(UiContainerKind::MasonryBox(
        UiMasonryBoxConfig {
            columns: 3,
            gap: 8.0,
            sequential: true,
        },
    ));
    let size_box =
        UiLayoutEngineRequest::from_container_kind(UiContainerKind::SizeBox(UiSizeBoxConfig {
            aspect_ratio: 1.0,
        }));
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
    assert_eq!(masonry.family, UiLayoutEngineFamily::Masonry);
    assert!(masonry.requires_zircon_semantics());
    assert_eq!(size_box.family, UiLayoutEngineFamily::Container);
    assert!(size_box.requires_zircon_semantics());
    assert_eq!(scrollable.family, UiLayoutEngineFamily::VirtualizedList);
    assert!(scrollable.requires_zircon_semantics());
    assert_eq!(round_trip(&scrollable), scrollable);
    assert_eq!(
        round_trip(&UiLayoutEngineFallbackReason::SlotFramePolicy),
        UiLayoutEngineFallbackReason::SlotFramePolicy
    );
    assert_eq!(
        round_trip(&UiLayoutEngineFallbackReason::AxisConstraintPriority),
        UiLayoutEngineFallbackReason::AxisConstraintPriority
    );
    assert_eq!(
        round_trip(&UiLayoutEngineFallbackReason::InvalidLayoutValue),
        UiLayoutEngineFallbackReason::InvalidLayoutValue
    );
}

#[test]
fn ui_layout_engine_block_is_explicit_not_implied_by_current_container_contracts() {
    let current_containers = [
        UiContainerKind::Free,
        UiContainerKind::Container,
        UiContainerKind::Overlay,
        UiContainerKind::Space,
        UiContainerKind::SizeBox(UiSizeBoxConfig { aspect_ratio: 1.0 }),
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 1.0 }),
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 1.0 }),
        UiContainerKind::ScrollableBox(UiScrollableBoxConfig::default()),
        UiContainerKind::WrapBox(Default::default()),
        UiContainerKind::GridBox(Default::default()),
        UiContainerKind::MasonryBox(Default::default()),
    ];

    for container in current_containers {
        assert_ne!(
            UiLayoutEngineRequest::from_container_kind(container).family,
            UiLayoutEngineFamily::Block
        );
    }

    let taffy = UiLayoutEngineCapability::taffy_flex_grid_block();
    let legacy = UiLayoutEngineCapability::legacy_zircon();
    let block = UiLayoutEngineRequest::new(UiLayoutEngineFamily::Block);
    let selection = UiLayoutEngineSelection::select(&block, &taffy, &legacy);
    assert_eq!(selection.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(selection.support, UiLayoutEngineSupport::Native);
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
    let taffy_flex = taffy_flex.with_node_id(crate::ui::event_ui::UiNodeId::new(7));
    let report = UiLayoutEngineSelectionReport::from_selections(vec![
        taffy_flex.clone(),
        taffy_overlay.clone(),
        taffy_scrollable.clone(),
    ]);

    assert_eq!(taffy_flex.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(
        taffy_flex.node_id,
        Some(crate::ui::event_ui::UiNodeId::new(7))
    );
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
    assert_eq!(report.fallback_reason_counts.len(), 1);
    assert_eq!(
        report.fallback_reason_counts[0].reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
    assert_eq!(report.fallback_reason_counts[0].count, 2);
    assert_eq!(report.taffy_tree_build_count, 0);
    assert_eq!(report.taffy_tree_node_count, 0);
    assert_eq!(round_trip(&report), report);
}

#[test]
fn ui_layout_engine_selection_report_tracks_taffy_tree_build_stats() {
    let taffy = UiLayoutEngineCapability::taffy_flex_grid_block();
    let legacy = UiLayoutEngineCapability::legacy_zircon();
    let selection = UiLayoutEngineSelection::select(
        &UiLayoutEngineRequest::new(UiLayoutEngineFamily::Flex),
        &taffy,
        &legacy,
    )
    .with_node_id(crate::ui::event_ui::UiNodeId::new(9))
    .with_taffy_tree_build(UiLayoutEngineTaffyTreeBuildStats::new(4));
    let report = UiLayoutEngineSelectionReport::from_selections(vec![selection.clone()]);

    assert_eq!(
        selection.taffy_tree_build,
        Some(UiLayoutEngineTaffyTreeBuildStats::new(4))
    );
    assert_eq!(report.request_count, 1);
    assert_eq!(report.taffy_selected_count, 1);
    assert_eq!(report.taffy_tree_build_count, 1);
    assert_eq!(report.taffy_tree_node_count, 4);
    assert_eq!(round_trip(&report), report);
}

#[test]
fn ui_layout_engine_selection_report_counts_unsupported_routes_separately() {
    let preferred = UiLayoutEngineCapability {
        backend: UiLayoutEngineBackend::Taffy,
        supported_families: Vec::new(),
        supports_content_measure: true,
        supports_dpi_scaling: true,
    };
    let fallback = UiLayoutEngineCapability {
        backend: UiLayoutEngineBackend::LegacyZircon,
        supported_families: Vec::new(),
        supports_content_measure: true,
        supports_dpi_scaling: true,
    };
    let request = UiLayoutEngineRequest::new(UiLayoutEngineFamily::Block);
    let selection = UiLayoutEngineSelection::select(&request, &preferred, &fallback)
        .with_node_id(crate::ui::event_ui::UiNodeId::new(42));
    let report = UiLayoutEngineSelectionReport::from_selections(vec![selection.clone()]);

    assert_eq!(selection.request.family, UiLayoutEngineFamily::Block);
    assert_eq!(selection.requested_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(
        selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(selection.support, UiLayoutEngineSupport::Unsupported);
    assert_eq!(
        selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::UnsupportedFamily)
    );
    assert_eq!(report.request_count, 1);
    assert_eq!(report.fallback_count, 0);
    assert_eq!(report.unsupported_count, 1);
    assert_eq!(report.fallback_reason_counts.len(), 1);
    assert_eq!(
        report.fallback_reason_counts[0].reason,
        Some(UiLayoutEngineFallbackReason::UnsupportedFamily)
    );
    assert_eq!(report.fallback_reason_counts[0].count, 1);
    assert_eq!(round_trip(&report), report);
}

#[test]
fn ui_layout_engine_selection_report_counts_missing_fallback_reasons() {
    let selection = UiLayoutEngineSelection {
        request: UiLayoutEngineRequest::new(UiLayoutEngineFamily::Overlay),
        requested_backend: UiLayoutEngineBackend::Taffy,
        selected_backend: UiLayoutEngineBackend::LegacyZircon,
        support: UiLayoutEngineSupport::Fallback,
        fallback_reason: None,
        ..UiLayoutEngineSelection::default()
    };
    let report = UiLayoutEngineSelectionReport::from_selections(vec![selection]);

    assert_eq!(report.request_count, 1);
    assert_eq!(report.fallback_count, 1);
    assert_eq!(report.unsupported_count, 0);
    assert_eq!(report.fallback_reason_counts.len(), 1);
    assert_eq!(report.fallback_reason_counts[0].reason, None);
    assert_eq!(report.fallback_reason_counts[0].count, 1);
    assert_eq!(round_trip(&report), report);
}

#[test]
fn ui_layout_engine_selection_report_deserialization_recomputes_aggregate_counts() {
    let stale_json = serde_json::json!({
        "selections": [
            {
                "request": {
                    "family": "flex",
                    "needs_content_measure": false,
                    "needs_dpi_scaling": true
                },
                "requested_backend": "taffy",
                "selected_backend": "taffy",
                "support": "native"
            },
            {
                "request": {
                    "family": "overlay",
                    "needs_content_measure": false,
                    "needs_dpi_scaling": true
                },
                "requested_backend": "taffy",
                "selected_backend": "legacy_zircon",
                "support": "fallback",
                "fallback_reason": "zircon_owned_semantics"
            }
        ],
        "request_count": 99,
        "taffy_selected_count": 0,
        "legacy_selected_count": 0,
        "fallback_count": 0,
        "unsupported_count": 7,
        "taffy_tree_build_count": 44,
        "taffy_tree_node_count": 99,
        "fallback_reason_counts": [
            { "reason": "unsupported_family", "count": 12 }
        ]
    });
    let report: UiLayoutEngineSelectionReport =
        serde_json::from_value(stale_json).expect("deserialize stale report");

    assert_eq!(report.request_count, 2);
    assert_eq!(report.taffy_selected_count, 1);
    assert_eq!(report.legacy_selected_count, 1);
    assert_eq!(report.fallback_count, 1);
    assert_eq!(report.unsupported_count, 0);
    assert_eq!(report.taffy_tree_build_count, 0);
    assert_eq!(report.taffy_tree_node_count, 0);
    assert_eq!(report.fallback_reason_counts.len(), 1);
    assert_eq!(
        report.fallback_reason_counts[0].reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
    assert_eq!(report.fallback_reason_counts[0].count, 1);

    let legacy_json_without_reason_counts = serde_json::json!({
        "selections": [
            {
                "request": {
                    "family": "overlay",
                    "needs_content_measure": false,
                    "needs_dpi_scaling": true
                },
                "requested_backend": "taffy",
                "selected_backend": "legacy_zircon",
                "support": "fallback",
                "fallback_reason": "zircon_owned_semantics"
            }
        ]
    });
    let legacy_report: UiLayoutEngineSelectionReport =
        serde_json::from_value(legacy_json_without_reason_counts)
            .expect("deserialize legacy report");

    assert_eq!(legacy_report.request_count, 1);
    assert_eq!(legacy_report.fallback_count, 1);
    assert_eq!(legacy_report.fallback_reason_counts.len(), 1);
    assert_eq!(
        legacy_report.fallback_reason_counts[0].reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
    assert_eq!(legacy_report.fallback_reason_counts[0].count, 1);
}
