use super::super::super::super::data::{HostTextInputFocusData, TemplatePaneNodeData};
use super::super::template_node_label;

#[test]
fn button_label_ignores_matching_text_input_focus_value() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserViewModeThumbButton".into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: "Thumb".into(),
        value_text: "thumbnail".into(),
        edit_action_id: "AssetSurface/SetViewMode".into(),
        ..TemplatePaneNodeData::default()
    };
    let focus = HostTextInputFocusData {
        control_id: "AssetBrowserViewModeThumbButton".into(),
        value_text: "thumbnail".into(),
        ..HostTextInputFocusData::default()
    };

    assert_eq!(template_node_label(&node, Some(&focus)), "Thumb");
}

#[test]
fn button_change_binding_label_prefers_authored_text_over_value() {
    let node = TemplatePaneNodeData {
        control_id: "AssetBrowserPreviewTabButton".into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: "Preview".into(),
        value_text: "preview".into(),
        edit_action_id: "AssetSurface/SetUtilityTab".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(template_node_label(&node, None), "Preview");
}

#[test]
fn input_label_uses_matching_text_input_focus_value() {
    let node = TemplatePaneNodeData {
        control_id: "SearchEdited".into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        text: "Search".into(),
        value_text: "mat".into(),
        ..TemplatePaneNodeData::default()
    };
    let focus = HostTextInputFocusData {
        control_id: "SearchEdited".into(),
        value_text: "material".into(),
        ..HostTextInputFocusData::default()
    };

    assert_eq!(template_node_label(&node, Some(&focus)), "material");
}
