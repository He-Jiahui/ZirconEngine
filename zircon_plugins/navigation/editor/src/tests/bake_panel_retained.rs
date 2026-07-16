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
            serde_json::json!([41, true])
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
            serde_json::json!([73])
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
        serde_json::json!([41, false])
    );

    panel.handle_retained_event(NavigationBakePanelEvent::SelectSurface(73));
    assert_eq!(
        operation(panel.handle_retained_event(NavigationBakePanelEvent::BakeSelectedClicked)).1,
        serde_json::json!([73, false])
    );
}
