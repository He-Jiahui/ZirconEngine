use std::{cell::RefCell, rc::Rc};

use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, KeyLocation, ModifiersState, NamedKey, PhysicalKey};

fn enter() -> KeyEvent {
    KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::Enter),
        logical_key: Key::Named(NamedKey::Enter),
        text: None,
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: false,
        text_with_all_modifiers: None,
        key_without_modifiers: Key::Named(NamedKey::Enter),
    }
}

fn focus(commit_action: &str, dispatch_kind: &str) -> HostTextInputFocusData {
    HostTextInputFocusData {
        control_id: "focused-test-setting".into(),
        dispatch_kind: dispatch_kind.into(),
        edit_action_id: "test-draft-only".into(),
        commit_action_id: commit_action.into(),
        value_text: "old draft 中文".into(),
        edit_frame: FrameRect {
            x: 12.0,
            y: 16.0,
            width: 160.0,
            height: 24.0,
        },
        ..HostTextInputFocusData::default()
    }
}

#[test]
fn replaced_settings_draft_cannot_receive_more_text_or_commit_on_enter() {
    let ui = UiHostWindow::new().expect("native host state");
    let commits = Rc::new(RefCell::new(Vec::new()));
    let received = commits.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_edited(move |control, binding, value| {
            received.borrow_mut().push((control, binding, value));
        });
    for (action, kind) in [
        (SETTINGS_COMMIT_STRING_ACTION_ID, "commit_only"),
        (SETTINGS_COMMIT_NUMERIC_ACTION_ID, "commit_only"),
        (SETTINGS_COMMIT_CHORD_ACTION_ID, "chord_capture"),
    ] {
        let original = focus(action, kind);
        let mut saved = Some(original.clone());
        ui.global::<UiHostContext>()
            .set_text_input_focus(original.clone());
        ui.dispatch_native_key_for_test(enter(), ModifiersState::empty());
        assert_eq!(commits.borrow().len(), 1);
        assert_eq!(commits.borrow()[0].1.as_str(), action);
        commits.borrow_mut().clear();
        ui.global::<UiHostContext>().set_text_input_focus(original);
        let before = ui.get_host_presentation_generation().cursor();
        assert!(retire_settings_native_draft(&ui, &mut saved));
        assert!(saved.is_none());
        assert!(!ui.text_input_focus_active());
        assert!(!ui
            .dispatch_focused_text_insert("late 中文")
            .request_redraw());
        ui.dispatch_native_key_for_test(enter(), ModifiersState::empty());
        assert!(commits.borrow().is_empty());
        assert!(before.same_business_generation(ui.get_host_presentation_generation().cursor()));
        assert!(!retire_settings_native_draft(&ui, &mut saved));
    }
}

#[test]
fn retirement_keeps_unrelated_focus_but_discards_a_saved_settings_draft() {
    let ui = UiHostWindow::new().expect("native host state");
    let received = Rc::new(RefCell::new(Vec::new()));
    let commits = received.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_edited(move |control, binding, value| {
            commits.borrow_mut().push((control, binding, value));
        });
    let original = focus("unrelated-search-commit", "commit_only");
    ui.global::<UiHostContext>()
        .set_text_input_focus(original.clone());
    let mut saved = Some(original.clone());
    let before = ui.get_host_presentation_generation().cursor();
    assert!(!retire_settings_native_draft(&ui, &mut saved));
    assert!(saved.as_ref() == Some(&original));
    assert!(ui.global::<UiHostContext>().get_text_input_focus() == original);
    assert!(before == ui.get_host_presentation_generation().cursor());

    saved = Some(focus(SETTINGS_COMMIT_NUMERIC_ACTION_ID, "commit_only"));
    assert!(retire_settings_native_draft(&ui, &mut saved));
    assert!(saved.is_none());
    assert!(ui.global::<UiHostContext>().get_text_input_focus() == original);
    ui.dispatch_native_key_for_test(enter(), ModifiersState::empty());
    let received = received.borrow();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].1.as_str(), "unrelated-search-commit");
    assert_eq!(received[0].2.as_str(), original.value_text.as_str());
}
