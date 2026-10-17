use super::*;
use crate::scene::viewport::{DisplayMode, ProjectionMode, TransformSpace, ViewOrientation};
use crate::ui::binding::ViewportCommand;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn actual_source_dropdowns_dispatch_selected_enum_and_numeric_options() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    for (control_id, option_id, expected) in [
        (
            "SetProjectionMode",
            "Orthographic",
            ViewportCommand::SetProjectionMode(ProjectionMode::Orthographic),
        ),
        (
            "SetDisplayMode",
            "WireOnly",
            ViewportCommand::SetDisplayMode(DisplayMode::WireOnly),
        ),
        (
            "SetGridMode",
            "Hidden",
            ViewportCommand::SetGridVisibility(false),
        ),
        (
            "SetTransformSpace",
            "Global",
            ViewportCommand::SetTransformSpace(TransformSpace::Global),
        ),
        (
            "AlignView",
            "NegZ",
            ViewportCommand::AlignView(ViewOrientation::NegZ),
        ),
        (
            "SetTranslateSnap",
            "2.0",
            ViewportCommand::SetTranslateSnap(2.0),
        ),
        (
            "SetRotateSnapDegrees",
            "30.0",
            ViewportCommand::SetRotateSnapDegrees(30.0),
        ),
        ("SetScaleSnap", "0.25", ViewportCommand::SetScaleSnap(0.25)),
    ] {
        let binding = bridge
            .binding_for_control(control_id, UiEventKind::Change)
            .unwrap()
            .clone();
        assert!(bridge
            .select_dropdown_option(control_id, option_id)
            .unwrap());
        let rebound = viewport_dropdown_binding_for_option(&binding, option_id).unwrap();
        assert_eq!(rebound.path(), binding.path());
        assert_eq!(
            rebound.payload(),
            &EditorUiBindingPayload::viewport_command(expected)
        );
    }
}

#[test]
fn source_rejects_unknown_or_disabled_options_and_typed_numeric_rebind_rejects_invalid_values() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    assert!(!bridge
        .select_dropdown_option("SetProjectionMode", "unknown")
        .unwrap());
    bridge
        .mutate_control_property_for_test(
            "SetProjectionMode",
            "disabled_options",
            zircon_runtime_interface::ui::component::UiValue::Array(vec![
                zircon_runtime_interface::ui::component::UiValue::String("Orthographic".into()),
            ]),
        )
        .unwrap();
    assert!(!bridge
        .select_dropdown_option("SetProjectionMode", "Orthographic")
        .unwrap());
    let binding = bridge
        .binding_for_control("SetTranslateSnap", UiEventKind::Change)
        .unwrap();
    for invalid in ["NaN", "inf", "-1", "0", "not a number"] {
        assert!(viewport_dropdown_binding_for_option(binding, invalid).is_err());
    }
}

#[test]
fn non_viewport_option_binding_keeps_existing_payload() {
    let binding = EditorUiBinding::new(
        "menu",
        "choice",
        UiEventKind::Change,
        EditorUiBindingPayload::menu_action("workbench.unchanged"),
    );
    let rebound = viewport_dropdown_binding_for_option(&binding, "different").unwrap();
    assert_eq!(rebound.as_ui_binding(), binding.as_ui_binding());
}
