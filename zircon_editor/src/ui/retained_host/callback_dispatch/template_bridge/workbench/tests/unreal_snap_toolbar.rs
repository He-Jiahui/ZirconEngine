use super::*;
use crate::scene::viewport::GridMode;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, ViewportCommand};
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

#[test]
fn unreal_mounted_snap_icons_use_resolved_booleans_and_keep_distinct_typed_actions() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let mut chrome = SceneViewportChromeSettings::default();
    chrome.grid_mode = GridMode::Hidden;
    chrome.translate_snap_enabled = false;
    chrome.rotate_snap_enabled = true;
    chrome.scale_snap_enabled = false;
    bridge.sync_unreal_viewport_toolbar_state(&chrome).unwrap();
    bridge.template_surface.refresh_after_state_change(bridge.runtime.as_ref()).unwrap();
    for (control_id, kind, checked) in [
        ("ToggleTranslateSnap", TransformHandleKind::Move, false),
        ("ToggleRotateSnap", TransformHandleKind::Rotate, true),
        ("ToggleScaleSnap", TransformHandleKind::Scale, false),
    ] {
        assert_eq!(bridge.control_bool(control_id, "checked"), checked);
        assert_eq!(bridge.control_bool(control_id, "selected"), checked);
        let binding = bridge.binding_for_control(control_id, UiEventKind::Click).unwrap();
        assert_eq!(binding.payload(), &EditorUiBindingPayload::viewport_command(ViewportCommand::ToggleSnap(kind)));
        assert_eq!(EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap().payload(), binding.payload());
    }
    let grid = bridge.binding_for_control("SetGridMode", UiEventKind::Change).unwrap();
    assert_eq!(grid.payload(), &EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridVisibility(true)));
    assert_eq!(bridge.control_string("SetTranslateSnap", "value_text").as_deref(), Some("1"));
}
