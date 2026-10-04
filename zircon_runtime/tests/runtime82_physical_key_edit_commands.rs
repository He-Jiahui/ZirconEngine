#![cfg(feature = "ui")]

use zircon_runtime::ui::{dispatch::UiInputManager, surface::UiSurface};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    text::UiTextEditReceipt,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract, UiWidgetEvent},
};

const INPUT: UiNodeId = UiNodeId::new(2);
const TEXT: &str = "alpha beta";

#[derive(Clone, Copy)]
enum PrimaryModifier {
    Control,
    Super,
}

#[test]
fn runtime82_physical_ctrl_arrows_move_by_word_without_selecting_all() {
    for state in [
        UiKeyboardInputState::Pressed,
        UiKeyboardInputState::Repeated,
    ] {
        for (key_code, initial, expected) in [(37, 10, 6), (39, 0, 5)] {
            let mut surface = editable_surface(initial, false);
            let mut manager = UiInputManager::default();

            let result = key(
                &mut surface,
                &mut manager,
                "",
                key_code,
                PrimaryModifier::Control,
                false,
                state,
            );

            assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
            assert_eq!(body(&surface), TEXT);
            assert_caret_selection(&surface, expected, expected, expected);
            assert!(edit_receipt(&result).is_none());
        }
    }
}

#[test]
fn runtime82_physical_ctrl_shift_arrow_extends_only_to_the_word_boundary() {
    let mut surface = editable_surface(10, false);
    let mut manager = UiInputManager::default();

    let result = key(
        &mut surface,
        &mut manager,
        "",
        37,
        PrimaryModifier::Control,
        true,
        UiKeyboardInputState::Pressed,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(body(&surface), TEXT);
    assert_caret_selection(&surface, 6, 10, 6);
    assert!(edit_receipt(&result).is_none());
}

#[test]
fn runtime82_physical_super_arrows_keep_line_navigation() {
    for (key_code, initial, expected) in [(37, 10, 0), (39, 0, 10)] {
        let mut surface = editable_surface(initial, false);
        let mut manager = UiInputManager::default();

        let result = key(
            &mut surface,
            &mut manager,
            "",
            key_code,
            PrimaryModifier::Super,
            false,
            UiKeyboardInputState::Pressed,
        );

        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(body(&surface), TEXT);
        assert_caret_selection(&surface, expected, expected, expected);
        assert!(edit_receipt(&result).is_none());
    }
}

#[test]
fn runtime82_physical_ctrl_word_deletion_is_one_undoable_document_edit() {
    for (key_code, initial) in [(8, 10), (46, 6)] {
        let mut surface = editable_surface(initial, false);
        let mut manager = UiInputManager::default();

        let result = key(
            &mut surface,
            &mut manager,
            "",
            key_code,
            PrimaryModifier::Control,
            false,
            UiKeyboardInputState::Pressed,
        );

        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(body(&surface), "alpha ");
        assert_caret_selection(&surface, 6, 6, 6);
        let deletion =
            edit_receipt(&result).expect("physical word deletion commits a document edit");
        assert_eq!(deletion.previous_revision.get(), 0);
        assert_eq!(deletion.revision.get(), 1);
        assert_eq!(
            (
                deletion.changed.old.start_byte,
                deletion.changed.old.end_byte
            ),
            (6, 10)
        );
        assert_eq!(
            (
                deletion.changed.new.start_byte,
                deletion.changed.new.end_byte
            ),
            (6, 6)
        );

        let undo = key(
            &mut surface,
            &mut manager,
            "z",
            90,
            PrimaryModifier::Control,
            false,
            UiKeyboardInputState::Pressed,
        );
        let undo = edit_receipt(&undo).expect("word deletion remains undoable");
        assert_eq!(body(&surface), TEXT);
        assert_eq!(undo.document_id, deletion.document_id);
        assert_eq!(undo.previous_revision, deletion.revision);
        assert_eq!(undo.revision.get(), 2);
    }
}

#[test]
fn runtime82_logical_and_physical_a_keep_primary_select_all() {
    for (logical_key, key_code, primary) in [
        ("a", 0, PrimaryModifier::Control),
        ("A", 0, PrimaryModifier::Control),
        ("", 65, PrimaryModifier::Control),
        ("", 97, PrimaryModifier::Control),
        ("a", 65, PrimaryModifier::Super),
        ("", 65, PrimaryModifier::Super),
    ] {
        let mut surface = editable_surface(4, false);
        let mut manager = UiInputManager::default();

        let result = key(
            &mut surface,
            &mut manager,
            logical_key,
            key_code,
            primary,
            false,
            UiKeyboardInputState::Pressed,
        );

        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(body(&surface), TEXT);
        assert_caret_selection(&surface, 10, 0, 10);
        assert!(edit_receipt(&result).is_none());
    }
}

#[test]
fn runtime82_unknown_physical_primary_key_does_not_select_all() {
    for primary in [PrimaryModifier::Control, PrimaryModifier::Super] {
        let mut surface = editable_surface(4, false);
        let mut manager = UiInputManager::default();

        let result = key(
            &mut surface,
            &mut manager,
            "",
            0,
            primary,
            false,
            UiKeyboardInputState::Pressed,
        );

        assert_eq!(body(&surface), TEXT);
        assert_caret_selection(&surface, 4, 4, 4);
        assert!(edit_receipt(&result).is_none());
    }
}

#[test]
fn runtime82_physical_secure_ctrl_commands_keep_line_boundary_policy() {
    for (key_code, expected_text) in [(37, TEXT), (8, "")] {
        let mut surface = editable_surface(10, true);
        let mut manager = UiInputManager::default();

        let result = key(
            &mut surface,
            &mut manager,
            "",
            key_code,
            PrimaryModifier::Control,
            false,
            UiKeyboardInputState::Pressed,
        );

        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(body(&surface), expected_text);
        assert_caret_selection(&surface, 0, 0, 0);
        assert!(result.diagnostics.secure_text_redacted);
    }
}

#[test]
fn runtime82_physical_command_release_does_not_edit_or_select() {
    let mut surface = editable_surface(4, false);
    let mut manager = UiInputManager::default();

    let result = key(
        &mut surface,
        &mut manager,
        "",
        8,
        PrimaryModifier::Control,
        false,
        UiKeyboardInputState::Released,
    );

    assert_eq!(body(&surface), TEXT);
    assert_caret_selection(&surface, 4, 4, 4);
    assert!(edit_receipt(&result).is_none());
}

fn editable_surface(caret: usize, secure: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.physical_key_commands.regression"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 80.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                ..UiStateFlags::default()
            }),
    );
    let mut attributes = [
        ("content".to_string(), toml::Value::String(TEXT.to_string())),
        (
            "caret_offset".to_string(),
            toml::Value::Integer(caret as i64),
        ),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    if secure {
        attributes.insert(
            "input_kind".to_string(),
            toml::Value::String("password".to_string()),
        );
    }
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(INPUT, UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(8.0, 8.0, 180.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    ..UiStateFlags::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_string(),
                    attributes,
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .expect("attach text input");
    surface.rebuild();
    surface.focus_node(INPUT).expect("focus text input");
    surface
}

fn key(
    surface: &mut UiSurface,
    manager: &mut UiInputManager,
    logical_key: &str,
    key_code: u32,
    primary: PrimaryModifier,
    shift: bool,
    state: UiKeyboardInputState,
) -> UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
    metadata.modifiers.control = matches!(primary, PrimaryModifier::Control);
    metadata.modifiers.super_key = matches!(primary, PrimaryModifier::Super);
    metadata.modifiers.shift = shift;
    surface
        .dispatch_input_event_with_manager(
            manager,
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata,
                state,
                key_code,
                scan_code: None,
                physical_key: format!("Key{key_code}"),
                logical_key: logical_key.to_string(),
                text: None,
            }),
        )
        .expect("dispatch physical or logical text keyboard command")
}

fn attributes(surface: &UiSurface) -> &std::collections::BTreeMap<String, toml::Value> {
    &surface
        .tree
        .node(INPUT)
        .and_then(|node| node.template_metadata.as_ref())
        .expect("editable input metadata")
        .attributes
}

fn body(surface: &UiSurface) -> &str {
    attributes(surface)
        .get("content")
        .and_then(toml::Value::as_str)
        .expect("editable body")
}

fn assert_caret_selection(surface: &UiSurface, caret: i64, anchor: i64, focus: i64) {
    let attributes = attributes(surface);
    let actual_caret = attributes
        .get("caret_offset")
        .and_then(toml::Value::as_integer)
        .expect("editable caret");
    assert_eq!(actual_caret, caret);
    assert_eq!(
        attributes
            .get("selection_anchor")
            .and_then(toml::Value::as_integer)
            .unwrap_or(actual_caret),
        anchor
    );
    assert_eq!(
        attributes
            .get("selection_focus")
            .and_then(toml::Value::as_integer)
            .unwrap_or(actual_caret),
        focus
    );
}

fn edit_receipt(result: &UiInputDispatchResult) -> Option<&UiTextEditReceipt> {
    result.widget_events.iter().find_map(|event| match event {
        UiWidgetEvent::TextEditChange { receipt } => Some(receipt.as_ref()),
        _ => None,
    })
}
