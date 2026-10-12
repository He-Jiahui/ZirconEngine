use super::*;

#[test]
fn runtime_pointer_focus_remains_semantic_without_drawing_keyboard_focus() {
    let node = TemplatePaneNodeData {
        focused: true,
        focus_visible: false,
        focus_visible_known: true,
        ..TemplatePaneNodeData::default()
    };

    let state = resolved_state_for_node(&node);

    assert!(state.focused);
    assert!(!state.focus_visible);
}

#[test]
fn runtime_keyboard_focus_and_static_preview_keep_visible_focus() {
    let keyboard = TemplatePaneNodeData {
        focused: true,
        focus_visible: true,
        focus_visible_known: true,
        ..TemplatePaneNodeData::default()
    };
    let static_preview = TemplatePaneNodeData {
        focus_visible: true,
        ..TemplatePaneNodeData::default()
    };

    assert!(resolved_state_for_node(&keyboard).focus_visible);
    assert!(resolved_state_for_node(&static_preview).focus_visible);
}
