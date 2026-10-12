use super::*;
use zircon_runtime_interface::ui::layout::UiSize;

const SEARCH_CONTROL: &str = "WorkbenchInputSearch";

#[test]
fn component_lab_numeric_edit_updates_value_text_and_normalized_position() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_eq!(
        bridge
            .edit_component_lab_field("WorkbenchInputSlider", "ComponentLab/InputSliderEdit", "25",)
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        bridge.control_float("WorkbenchInputSlider", "value"),
        Some(25.0)
    );
    assert_eq!(
        bridge.control_float("WorkbenchInputSlider", "value_percent"),
        Some(0.25)
    );
    assert_eq!(
        bridge
            .control_string("WorkbenchInputSlider", "value_text")
            .as_deref(),
        Some("25")
    );
}

#[test]
fn component_lab_search_edit_updates_its_declared_value_property() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_eq!(
        bridge
            .edit_component_lab_field(SEARCH_CONTROL, "ComponentLab/InputSearchEdit", "button",)
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        bridge.control_string(SEARCH_CONTROL, "query").as_deref(),
        Some("button")
    );
}

#[test]
fn component_lab_text_edit_preserves_user_whitespace() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_eq!(
        bridge
            .edit_component_lab_field(
                "WorkbenchInputText",
                "ComponentLab/InputTextEdit",
                "  exact text  ",
            )
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        bridge
            .control_string("WorkbenchInputText", "value")
            .as_deref(),
        Some("  exact text  ")
    );
}

#[test]
fn component_lab_incomplete_numeric_draft_keeps_the_last_valid_value() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_eq!(
        bridge
            .edit_component_lab_field(
                "WorkbenchInputStepper",
                "ComponentLab/InputStepperEdit",
                "-",
            )
            .unwrap(),
        Some(true)
    );
    assert_eq!(
        bridge.control_float("WorkbenchInputStepper", "value"),
        Some(42.0)
    );
    assert_eq!(
        bridge
            .control_string("WorkbenchInputStepper", "value_text")
            .as_deref(),
        Some("-")
    );
}
