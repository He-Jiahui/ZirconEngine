use super::*;

#[test]
fn mounted_layout_reports_only_size_or_scale_changes() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();

    let (_, stable) =
        bridge.prepare_layout_at_mount_with_scale(UiFrame::new(0.0, 0.0, 900.0, 620.0), 1.0);
    let (_, origin_only) =
        bridge.prepare_layout_at_mount_with_scale(UiFrame::new(25.0, 40.0, 900.0, 620.0), 1.0);
    let (_, resized) =
        bridge.prepare_layout_at_mount_with_scale(UiFrame::new(25.0, 40.0, 901.0, 620.0), 1.0);
    let (_, rescaled) =
        bridge.prepare_layout_at_mount_with_scale(UiFrame::new(25.0, 40.0, 901.0, 620.0), 1.25);

    assert!(!stable);
    assert!(!origin_only);
    assert!(resized);
    assert!(rescaled);
}
