use super::*;
use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextEditAction, UiTextSelection,
};

#[test]
fn render_extract_outputs_editable_text_state_for_text_fields() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/input"))
            .with_frame(UiFrame::new(4.0, 8.0, 96.0, 24.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "TextField".to_string(),
                control_id: Some("EditableDemo".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
value = "Hello"
font_size = 10.0
line_height = 12.0
focused = true
caret_offset = 4
caret_affinity = "upstream"
selection_anchor = 1
selection_focus = 4
composition_start = 1
composition_end = 4
composition_text = "ell"
read_only = true
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let layout = first_text_layout(&surface);
    let editable = layout
        .editable
        .as_ref()
        .expect("TextField render layout should carry editable state");
    assert_eq!(editable.text, "Hello");
    assert_eq!(editable.caret.offset, 4);
    assert_eq!(editable.caret.affinity, UiTextCaretAffinity::Upstream);
    assert_eq!(
        editable.selection,
        Some(UiTextSelection {
            anchor: 1,
            focus: 4,
        })
    );
    assert_eq!(editable.composition.as_ref().unwrap().range.start, 1);
    assert_eq!(editable.composition.as_ref().unwrap().range.end, 4);
    assert_eq!(editable.composition.as_ref().unwrap().text, "ell");
    assert!(editable
        .composition
        .as_ref()
        .unwrap()
        .restore_text
        .is_none());
    assert!(editable.read_only);
}

#[test]
fn render_extract_injects_preedit_span_without_document_value_mutation() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/input"))
            .with_frame(UiFrame::new(4.0, 8.0, 120.0, 24.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "TextField".to_string(),
                control_id: Some("PreeditLayoutDemo".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
value = "Hello"
font_size = 10.0
line_height = 12.0
focused = true
composition_start = 1
composition_end = 4
composition_text = "拼"
composition_restore_text = "ell"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
                ..Default::default()
            }),
    );

    surface.rebuild();

    let command = first_text_layout_command(&surface);
    assert_eq!(command.text.as_deref(), Some("Hello"));

    let layout = command.text_layout.as_ref().unwrap();
    assert_eq!(layout.lines[0].text, "H拼o");
    let editable = layout.editable.as_ref().unwrap();
    assert_eq!(editable.text, "Hello");
    assert_eq!(editable.composition.as_ref().unwrap().range.start, 1);
    assert_eq!(editable.composition.as_ref().unwrap().range.end, 4);
    assert_eq!(editable.composition.as_ref().unwrap().text, "拼");
    assert_eq!(
        editable
            .composition
            .as_ref()
            .unwrap()
            .restore_text
            .as_deref(),
        Some("ell")
    );
}

#[test]
fn editable_text_state_applies_selection_and_composition_actions() {
    let state = UiEditableTextState {
        text: "Hello".to_string(),
        caret: UiTextCaret {
            offset: 5,
            affinity: Default::default(),
        },
        selection: Some(UiTextSelection {
            anchor: 1,
            focus: 4,
        }),
        composition: None,
        read_only: false,
    };

    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::Insert {
            text: "ey".to_string(),
        },
    );
    assert_eq!(state.text, "Heyo");
    assert_eq!(state.caret.offset, 3);
    assert_eq!(state.selection, None);

    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 3 },
            text: "allo".to_string(),
        },
    );
    let state = crate::ui::text::apply_text_edit_action(state, UiTextEditAction::CommitComposition);
    assert_eq!(state.text, "Hallo");
    assert_eq!(state.caret.offset, 5);
    assert_eq!(state.composition, None);
}

#[test]
fn editable_text_state_restores_preedit_text_when_composition_is_canceled() {
    let state = UiEditableTextState {
        text: "Hello".to_string(),
        caret: UiTextCaret {
            offset: 1,
            affinity: Default::default(),
        },
        selection: None,
        composition: None,
        read_only: false,
    };

    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 5 },
            text: "allo".to_string(),
        },
    );
    assert_eq!(state.text, "Hallo");
    assert!(state.composition.is_some());

    let state = crate::ui::text::apply_text_edit_action(state, UiTextEditAction::CancelComposition);
    assert_eq!(state.text, "Hello");
    assert_eq!(state.composition, None);
}

#[test]
fn editable_text_state_updates_composition_against_preedit_base_text() {
    let state = UiEditableTextState {
        text: "Hello".to_string(),
        caret: UiTextCaret {
            offset: 1,
            affinity: Default::default(),
        },
        selection: None,
        composition: None,
        read_only: false,
    };

    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 2 },
            text: "a".to_string(),
        },
    );
    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 2 },
            text: "al".to_string(),
        },
    );
    assert_eq!(state.text, "Hallo");

    let state = crate::ui::text::apply_text_edit_action(state, UiTextEditAction::CancelComposition);
    assert_eq!(state.text, "Hello");
    assert_eq!(state.composition, None);
}

#[test]
fn editable_text_state_inserts_preedit_without_consuming_text_for_empty_range() {
    let state = UiEditableTextState {
        text: "Hello".to_string(),
        caret: UiTextCaret {
            offset: 1,
            affinity: Default::default(),
        },
        selection: None,
        composition: None,
        read_only: false,
    };

    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 1 },
            text: "allo".to_string(),
        },
    );
    assert_eq!(state.text, "Halloello");

    let state = crate::ui::text::apply_text_edit_action(state, UiTextEditAction::CancelComposition);
    assert_eq!(state.text, "Hello");
    assert_eq!(state.composition, None);
}
