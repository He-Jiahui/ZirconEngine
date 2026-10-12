use super::*;
use crate::scene::viewport::{GridMode, ProjectionMode};
use crate::ui::binding::{EditorUiBindingPayload, ViewportCommand};
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

#[test]
fn mounted_mode_icons_keep_fixed_commands_and_follow_actual_mode() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let modes = [
        ("SelectMode", SceneModeActivation::Select),
        ("TranslateMode", SceneModeActivation::Transform(TransformHandleKind::Move)),
        ("RotateMode", SceneModeActivation::Transform(TransformHandleKind::Rotate)),
        ("ScaleMode", SceneModeActivation::Transform(TransformHandleKind::Scale)),
    ];
    for (active_control, active_mode) in &modes {
        let mut settings = SceneViewportChromeSettings::default();
        settings.mode = active_mode.clone();
        bridge.sync_unreal_viewport_toolbar_state(&settings).unwrap();
        bridge.template_surface.refresh_after_state_change(bridge.runtime.as_ref()).unwrap();
        for (control_id, expected_mode) in &modes {
            let selected = control_id == active_control;
            assert_eq!(bridge.control_bool(control_id, "selected"), selected);
            assert_eq!(bridge.control_bool(control_id, "checked"), selected);
            let binding = bridge.binding_for_control(control_id, UiEventKind::Click).unwrap();
            assert_eq!(binding.payload(), &EditorUiBindingPayload::viewport_command(
                ViewportCommand::ActivateSceneMode(expected_mode.clone())));
            assert!(bridge.host_projection().node_by_control_id(control_id).is_some());
        }
    }
}

#[test]
fn mounted_dropdown_values_and_short_captions_follow_resolved_chrome() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let mut settings = SceneViewportChromeSettings::default();
    settings.projection_mode = ProjectionMode::Orthographic;
    settings.display_mode = DisplayMode::WireOnly;
    settings.transform_space = TransformSpace::Global;
    settings.pivot_mode = PivotMode::Primary;
    settings.grid_mode = GridMode::Hidden;
    settings.view_orientation = ViewOrientation::NegZ;
    settings.translate_step = 2.0;
    settings.rotate_step_deg = 30.0;
    settings.scale_step = 0.25;
    bridge.sync_unreal_viewport_toolbar_state(&settings).unwrap();
    bridge.template_surface.refresh_after_state_change(bridge.runtime.as_ref()).unwrap();
    for (control_id, value, caption) in [
        ("SetProjectionMode", "Orthographic", "Orthographic"),
        ("SetDisplayMode", "WireOnly", "Wireframe"),
        ("SetTransformSpace", "Global", "World"),
        ("SetPivotMode", "Primary", "Pivot"),
        ("SetGridMode", "Hidden", "Show"),
        ("AlignView", "NegZ", "-Z"),
        ("SetTranslateSnap", "2.0", "2"),
        ("SetRotateSnapDegrees", "30.0", "30"),
        ("SetScaleSnap", "0.25", "0.25"),
    ] {
        assert_eq!(bridge.control_string(control_id, "value").as_deref(), Some(value));
        assert_eq!(bridge.control_string(control_id, "value_text").as_deref(), Some(caption));
        assert!(bridge.host_projection().node_by_control_id(control_id).is_some());
    }
    bridge.sync_unreal_viewport_toolbar_state(&SceneViewportChromeSettings::default()).unwrap();
    assert_eq!(bridge.control_string("SetTranslateSnap", "value_text").as_deref(), Some("1"));
    assert_eq!(bridge.control_string("SetDisplayMode", "value_text").as_deref(), Some("Lit"));
}
