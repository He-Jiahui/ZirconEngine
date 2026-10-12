use super::*;

#[test]
fn reserved_input_metadata_preserves_arrival_identity_when_window_id_is_attached() {
    let ui = UiHostWindow::new().expect("root host window");
    let mut metadata = native_input_metadata_without_window_id(41);
    let timestamp = metadata.timestamp;
    let sequence = metadata.sequence;

    assert!(metadata.window_id.is_none());
    attach_native_window_id(&ui, &mut metadata);

    assert_eq!(metadata.timestamp, timestamp);
    assert_eq!(metadata.sequence, sequence);
    assert_eq!(
        metadata.window_id,
        Some(UiWindowId::new(NATIVE_HOST_WINDOW_ID))
    );
}

#[test]
fn child_input_metadata_uses_its_floating_window_identity() {
    let ui = UiHostWindow::new().expect("child host window");
    ui.set_native_floating_window_presentation(
        "window:hierarchy",
        "tree:hierarchy",
        "Hierarchy",
        &crate::ui::retained_host::host_contract::data::FrameRect::default(),
    );
    let metadata = native_input_metadata(&ui, 7);
    assert_eq!(
        metadata.window_id,
        Some(UiWindowId::new("window:hierarchy"))
    );
    assert_eq!(metadata.sequence, UiInputSequence::new(7));
}
