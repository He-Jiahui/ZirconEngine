use super::*;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn toolbar_priority_uses_logical_width_under_scale() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 840.0))
        .expect("scaled workbench bridge should build");

    bridge
        .recompute_layout_with_workbench_model_at_scale(
            UiSize::new(1280.0, 840.0),
            2.0,
            &model,
            &metrics,
        )
        .expect("logical 640px workbench should recompute");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchToolbarToolGroup"),
        Some(UiVisibility::Collapsed),
        "1280 physical pixels at 2x should use the logical 640px toolbar priority"
    );
    assert!(bridge.control_frame("WorkbenchRunPlay").is_some());
    assert!(bridge.control_frame("WorkbenchRunMode").is_some());

    bridge
        .recompute_layout_with_workbench_model_at_scale(
            UiSize::new(2520.0, 1560.0),
            2.0,
            &model,
            &metrics,
        )
        .expect("logical 1260px workbench should recompute");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchToolbarToolGroup"),
        Some(UiVisibility::Visible),
        "2520 physical pixels at 2x should use the logical 1260px toolbar priority"
    );
    for control_id in ["WorkbenchLayoutGrid", "WorkbenchThemeToggle"] {
        assert_eq!(
            control_visibility(&bridge, control_id),
            Some(UiVisibility::Collapsed),
            "logical 1260px should still defer {control_id}"
        );
    }
}
