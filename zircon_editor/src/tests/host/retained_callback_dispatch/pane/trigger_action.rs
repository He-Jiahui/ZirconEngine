use super::super::support::*;
#[test]
fn builtin_pane_surface_does_not_expose_the_retired_fixture_action() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_retained_empty_pane_surface");
    let bridge = BuiltinPaneSurfaceTemplateBridge::new().unwrap();
    let result = dispatch_builtin_pane_surface_control(
        &harness.runtime,
        &bridge,
        "TriggerAction",
        UiEventKind::Click,
        Vec::new(),
    );

    assert!(result.is_none());
    assert!(harness.runtime.journal().records().is_empty());
}
