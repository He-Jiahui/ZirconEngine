use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{
        UiEditableTextState, UiTextAlign, UiTextCaret, UiTextDirection, UiTextEditAction,
        UiTextOverflow, UiTextRange, UiTextRunKind, UiTextSelection, UiTextWrap,
    },
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_outputs_aligned_wrapped_text_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(10.0, 20.0, 60.0, 48.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("RuntimeLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha Beta Gamma Delta"
font_size = 10.0
line_height = 12.0
text_align = "center"
wrap = "word"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
            }),
    );

    surface.rebuild();

    let command = surface.render_extract.list.commands.first().unwrap();
    let layout = command
        .text_layout
        .as_ref()
        .expect("text command should carry resolved layout data");
    assert_eq!(layout.text_align, UiTextAlign::Center);
    assert_eq!(layout.wrap, UiTextWrap::Word);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "Alpha Beta");
    assert_eq!(layout.lines[0].frame, UiFrame::new(15.0, 20.0, 50.0, 12.0));
    assert_eq!(layout.lines[0].source_range.start, 0);
    assert_eq!(layout.lines[0].runs[0].kind, UiTextRunKind::Plain);
    assert_eq!(layout.lines[1].text, "Gamma Delta");
    assert_eq!(layout.lines[1].frame, UiFrame::new(12.5, 32.0, 55.0, 12.0));
    assert!(!layout.overflow_clipped);
}

#[test]
fn render_extract_clips_text_layout_to_clip_frame() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 40.0, 48.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("ClippedLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha Beta Gamma"
font_size = 10.0
line_height = 12.0
wrap = "glyph"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
            }),
    );
    surface
        .tree
        .node_mut(UiNodeId::new(1))
        .unwrap()
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 40.0, 12.0));

    surface.rebuild();

    let command = surface.render_extract.list.commands.first().unwrap();
    let layout = command.text_layout.as_ref().unwrap();
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "Alpha Be");
    assert!(layout.overflow_clipped);
}

#[test]
fn render_extract_outputs_rich_directional_ellipsis_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 50.0, 12.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("RichLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "Alpha **Beta** שלום Gamma"
font_size = 10.0
line_height = 12.0
wrap = "word"
text_overflow = "ellipsis"
rich_text = true
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    assert_eq!(layout.direction, UiTextDirection::Mixed);
    assert_eq!(layout.overflow, UiTextOverflow::Ellipsis);
    assert_eq!(layout.lines.len(), 1);
    assert!(layout.lines[0].ellipsized);
    assert!(layout.lines[0].text.ends_with('…'));
    assert!(layout.lines[0]
        .runs
        .iter()
        .any(|run| run.kind == UiTextRunKind::Strong));
}

#[test]
fn render_extract_outputs_visual_order_ranges_for_mixed_direction_text() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 16.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("MixedDirectionLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abc שלום def"
font_size = 10.0
line_height = 12.0
wrap = "none"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    let line = &layout.lines[0];
    assert_eq!(layout.direction, UiTextDirection::Mixed);
    assert_eq!(line.direction, UiTextDirection::Mixed);
    assert_eq!(line.text, "abc םולש def");
    assert_eq!(line.source_range, UiTextRange { start: 0, end: 16 });
    assert_eq!(line.visual_range, UiTextRange { start: 0, end: 16 });
    assert_eq!(line.runs.len(), 3);
    assert_eq!(line.runs[0].text, "abc ");
    assert_eq!(line.runs[0].source_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].visual_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].direction, UiTextDirection::LeftToRight);
    assert_eq!(line.runs[1].text, "םולש");
    assert_eq!(line.runs[1].source_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(line.runs[1].visual_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(line.runs[1].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[2].text, " def");
    assert_eq!(
        line.runs[2].source_range,
        UiTextRange { start: 12, end: 16 }
    );
    assert_eq!(
        line.runs[2].visual_range,
        UiTextRange { start: 12, end: 16 }
    );
    assert_eq!(line.runs[2].direction, UiTextDirection::LeftToRight);
}

#[test]
fn render_extract_keeps_neutral_separator_inside_rtl_visual_span() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 16.0))
            .with_state_flags(visible_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                control_id: Some("MixedDirectionNeutralLabel".to_string()),
                classes: Vec::new(),
                attributes: toml::from_str(
                    r#"
text = "abc שלום-עולם def"
font_size = 10.0
line_height = 12.0
wrap = "none"
"#,
                )
                .unwrap(),
                slot_attributes: Default::default(),
                style_overrides: Default::default(),
                style_tokens: Default::default(),
                bindings: Vec::new(),
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    let line = &layout.lines[0];
    assert_eq!(layout.direction, UiTextDirection::Mixed);
    assert_eq!(line.text, "abc םלוע-םולש def");
    assert_eq!(line.source_range, UiTextRange { start: 0, end: 25 });
    assert_eq!(line.visual_range, UiTextRange { start: 0, end: 25 });
    assert_eq!(line.runs.len(), 5);
    assert_eq!(line.runs[0].text, "abc ");
    assert_eq!(line.runs[0].source_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].visual_range, UiTextRange { start: 0, end: 4 });
    assert_eq!(line.runs[0].direction, UiTextDirection::LeftToRight);
    assert_eq!(line.runs[1].text, "םלוע");
    assert_eq!(
        line.runs[1].source_range,
        UiTextRange { start: 13, end: 21 }
    );
    assert_eq!(line.runs[1].visual_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(line.runs[1].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[2].text, "-");
    assert_eq!(
        line.runs[2].source_range,
        UiTextRange { start: 12, end: 13 }
    );
    assert_eq!(
        line.runs[2].visual_range,
        UiTextRange { start: 12, end: 13 }
    );
    assert_eq!(line.runs[2].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[3].text, "םולש");
    assert_eq!(line.runs[3].source_range, UiTextRange { start: 4, end: 12 });
    assert_eq!(
        line.runs[3].visual_range,
        UiTextRange { start: 13, end: 21 }
    );
    assert_eq!(line.runs[3].direction, UiTextDirection::RightToLeft);
    assert_eq!(line.runs[4].text, " def");
    assert_eq!(
        line.runs[4].source_range,
        UiTextRange { start: 21, end: 25 }
    );
    assert_eq!(
        line.runs[4].visual_range,
        UiTextRange { start: 21, end: 25 }
    );
    assert_eq!(line.runs[4].direction, UiTextDirection::LeftToRight);
}

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
caret_offset = 4
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
            }),
    );

    surface.rebuild();

    let layout = surface.render_extract.list.commands[0]
        .text_layout
        .as_ref()
        .unwrap();
    let editable = layout
        .editable
        .as_ref()
        .expect("TextField render layout should carry editable state");
    assert_eq!(editable.text, "Hello");
    assert_eq!(editable.caret.offset, 4);
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
            range: zircon_runtime_interface::ui::surface::UiTextRange { start: 1, end: 3 },
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
            range: zircon_runtime_interface::ui::surface::UiTextRange { start: 1, end: 5 },
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
            range: zircon_runtime_interface::ui::surface::UiTextRange { start: 1, end: 2 },
            text: "a".to_string(),
        },
    );
    let state = crate::ui::text::apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: zircon_runtime_interface::ui::surface::UiTextRange { start: 1, end: 2 },
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
            range: zircon_runtime_interface::ui::surface::UiTextRange { start: 1, end: 1 },
            text: "allo".to_string(),
        },
    );
    assert_eq!(state.text, "Halloello");

    let state = crate::ui::text::apply_text_edit_action(state, UiTextEditAction::CancelComposition);
    assert_eq!(state.text, "Hello");
    assert_eq!(state.composition, None);
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
