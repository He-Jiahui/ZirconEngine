#[test]
fn play_pointer_commands_use_the_navigation_only_entry_and_block_authoring_frame_selection() {
    let source = include_str!("../editor_state_viewport.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map_or(source, |(production, _)| production);

    assert!(production.contains("handle_play_viewport_navigation"));
    assert!(production.contains("ViewportCommand::FrameSelection if self.is_playing()"));
    assert!(production.contains("ViewportCommand::LeftPressed { .. }"));
    assert!(production.contains("ViewportCommand::LeftReleased"));
}
