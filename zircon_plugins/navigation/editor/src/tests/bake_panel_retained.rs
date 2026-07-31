use crate::bake_panel::{
    NavigationBakePanel, NavigationBakePanelEvent, NavigationBakePanelEventOutcome,
    NavigationBakeSelectionError, NavigationBakeSurfaceRow,
};
use zircon_runtime::core::framework::navigation::{
    NAVIGATION_BAKE_SURFACE_OPERATION, NAVIGATION_CLEAR_SURFACE_OPERATION,
};

fn surfaces() -> Vec<NavigationBakeSurfaceRow> {
    vec![
        NavigationBakeSurfaceRow::new(41, "Upper Deck"),
        NavigationBakeSurfaceRow::new(73, "Lower Deck"),
    ]
}

fn operation(outcome: NavigationBakePanelEventOutcome) -> (String, serde_json::Value) {
    let NavigationBakePanelEventOutcome::Operation(invocation) = outcome else {
        panic!("expected a retained navigation operation")
    };
    (invocation.operation_id.to_string(), invocation.arguments)
}

#[test]
fn navigation_bake_select_a_then_click_bake_submits_a() {
    let mut panel = NavigationBakePanel::default();
    panel.handle_retained_event(NavigationBakePanelEvent::ReplaceSurfaceRows(surfaces()));
    panel.handle_retained_event(NavigationBakePanelEvent::SelectSurface(41));
    panel.handle_retained_event(NavigationBakePanelEvent::ForceFullRebuildChanged(true));

    assert_eq!(
        operation(panel.handle_retained_event(NavigationBakePanelEvent::BakeSelectedClicked)),
        (
            NAVIGATION_BAKE_SURFACE_OPERATION.to_string(),
            serde_json::json!({ "surface_entity": 41, "force_full_rebuild": true })
        )
    );
}

#[test]
fn navigation_bake_select_b_then_click_clear_submits_b() {
    let mut panel = NavigationBakePanel::default();
    panel.handle_retained_event(NavigationBakePanelEvent::ReplaceSurfaceRows(surfaces()));
    panel.handle_retained_event(NavigationBakePanelEvent::SelectSurface(73));

    assert_eq!(
        operation(panel.handle_retained_event(NavigationBakePanelEvent::ClearSelectedClicked)),
        (
            NAVIGATION_CLEAR_SURFACE_OPERATION.to_string(),
            serde_json::json!({ "surface_entity": 73 })
        )
    );
}

#[test]
fn navigation_bake_without_selection_disables_selected_actions_and_submits_nothing() {
    let mut panel = NavigationBakePanel::default();
    panel.handle_retained_event(NavigationBakePanelEvent::ReplaceSurfaceRows(surfaces()));

    assert!(!panel.selected_actions_enabled());
    assert_eq!(
        panel.handle_retained_event(NavigationBakePanelEvent::BakeSelectedClicked),
        NavigationBakePanelEventOutcome::Ignored(NavigationBakeSelectionError::NoSurfaceSelected)
    );
    assert_eq!(
        panel.handle_retained_event(NavigationBakePanelEvent::ClearSelectedClicked),
        NavigationBakePanelEventOutcome::Ignored(NavigationBakeSelectionError::NoSurfaceSelected)
    );
}

#[test]
fn navigation_bake_switching_selection_rebuilds_arguments_without_stale_entity() {
    let mut panel = NavigationBakePanel::default();
    panel.handle_retained_event(NavigationBakePanelEvent::ReplaceSurfaceRows(surfaces()));
    panel.handle_retained_event(NavigationBakePanelEvent::SelectSurface(41));
    assert_eq!(
        operation(panel.handle_retained_event(NavigationBakePanelEvent::BakeSelectedClicked)).1,
        serde_json::json!({ "surface_entity": 41, "force_full_rebuild": false })
    );

    panel.handle_retained_event(NavigationBakePanelEvent::SelectSurface(73));
    assert_eq!(
        operation(panel.handle_retained_event(NavigationBakePanelEvent::BakeSelectedClicked)).1,
        serde_json::json!({ "surface_entity": 73, "force_full_rebuild": false })
    );
}

#[test]
fn navigation_bake_row_refresh_clears_an_unavailable_selected_surface() {
    let mut panel = NavigationBakePanel::default();
    panel.handle_retained_event(NavigationBakePanelEvent::ReplaceSurfaceRows(surfaces()));
    panel.handle_retained_event(NavigationBakePanelEvent::SelectSurface(41));
    panel.handle_retained_event(NavigationBakePanelEvent::ReplaceSurfaceRows(vec![
        NavigationBakeSurfaceRow::new(73, "Lower Deck"),
    ]));

    assert_eq!(panel.selected_surface_entity(), None);
    assert!(!panel.selected_actions_enabled());
    assert_eq!(
        panel.handle_retained_event(NavigationBakePanelEvent::BakeSelectedClicked),
        NavigationBakePanelEventOutcome::Ignored(NavigationBakeSelectionError::NoSurfaceSelected)
    );
    assert_eq!(
        panel.handle_retained_event(NavigationBakePanelEvent::ClearSelectedClicked),
        NavigationBakePanelEventOutcome::Ignored(NavigationBakeSelectionError::NoSurfaceSelected)
    );
}

#[test]
fn navigation_bake_template_projects_stable_selected_surface_arguments() {
    let document =
        zircon_runtime::ui::v2::UiV2AssetLoader::load_toml_str(include_str!("../../bake.zui"))
            .expect("navigation bake template should parse");

    assert_eq!(
        document
            .nodes
            .get("navigation_surface_list")
            .and_then(|node| node.props.get("selected_index"))
            .and_then(|value| value.as_integer()),
        Some(-1),
        "the surface list must start without a display-index selection"
    );

    let selected_binding = document
        .nodes
        .get("navigation_bake_selected_button")
        .and_then(|node| node.events.first())
        .expect("Bake Selected should declare a retained event binding");
    let selected_action = selected_binding
        .action
        .as_ref()
        .expect("Bake Selected should project operation arguments");
    assert_eq!(
        selected_action.route.as_deref(),
        Some(NAVIGATION_BAKE_SURFACE_OPERATION)
    );
    assert_eq!(
        selected_action
            .payload
            .get("surface_entity")
            .and_then(|value| value.as_str()),
        Some("=control.NavigationBakeSurfaceList.prop.selected_row_identity")
    );
    assert_eq!(
        selected_action
            .payload
            .get("force_full_rebuild")
            .and_then(|value| value.as_str()),
        Some("=control.NavigationForceFullRebuild.prop.checked")
    );

    let clear_binding = document
        .nodes
        .get("navigation_clear_button")
        .and_then(|node| node.events.first())
        .expect("Clear Selected should declare a retained event binding");
    let clear_action = clear_binding
        .action
        .as_ref()
        .expect("Clear Selected should project operation arguments");
    assert_eq!(
        clear_action.route.as_deref(),
        Some(NAVIGATION_CLEAR_SURFACE_OPERATION)
    );
    assert_eq!(
        clear_action
            .payload
            .get("surface_entity")
            .and_then(|value| value.as_str()),
        Some("=control.NavigationBakeSurfaceList.prop.selected_row_identity")
    );
    assert!(
        selected_action
            .payload
            .values()
            .chain(clear_action.payload.values())
            .all(|value| !value
                .as_str()
                .is_some_and(|value| value.contains("selected_index"))),
        "display indexes must never be submitted as Navigation surface entities"
    );
    for node_id in ["navigation_bake_selected_button", "navigation_clear_button"] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .and_then(|node| node.props.get("disabled"))
                .and_then(|value| value.as_str()),
            Some("=control.NavigationBakeSurfaceList.prop.selected_row_identity == null"),
            "selected-surface actions must be disabled without a row identity"
        );
    }
}
