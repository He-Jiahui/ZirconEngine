use super::*;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn workbench_progress_defaults_to_the_shared_accent_fill() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchSampleWeightsRunForward".into(),
        component_role: "progress".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        progress_fill_color(&node),
        material_feedback_palette().accent
    );
}

#[test]
fn semantic_progress_tone_overrides_the_normal_fill() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchBuildProgress".into(),
        component_role: "progress".into(),
        validation_level: "warning".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        progress_fill_color(&node),
        material_feedback_palette().warning
    );
}

#[test]
fn explicit_progress_fill_overrides_the_semantic_tone() {
    let mut node = TemplatePaneNodeData {
        control_id: "WorkbenchBuildProgress".into(),
        component_role: "progress".into(),
        validation_level: "warning".into(),
        ..TemplatePaneNodeData::default()
    };
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(11, 22, 33, 255)));

    assert_eq!(progress_fill_color(&node), [11, 22, 33, 255]);
}

#[test]
fn disabled_progress_colors_fall_back_to_the_shared_disabled_palette() {
    let node = TemplatePaneNodeData {
        control_id: "WorkbenchBuildProgress".into(),
        component_role: "progress".into(),
        disabled: true,
        ..TemplatePaneNodeData::default()
    };
    let palette = material_feedback_palette();

    assert_eq!(progress_track_color(&node), palette.disabled_track);
    assert_eq!(progress_fill_color(&node), palette.disabled_fill);
}
