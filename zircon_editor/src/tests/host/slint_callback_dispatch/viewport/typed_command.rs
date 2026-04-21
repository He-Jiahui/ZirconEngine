use super::super::support::*;

#[test]
fn typed_viewport_command_dispatch_updates_render_packet_without_pointer_bridge() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_viewport_command_dispatch");

    let effects = dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetDisplayMode(DisplayMode::WireOnly),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetGridMode(GridMode::VisibleAndSnap),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetProjectionMode(ProjectionMode::Orthographic),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetTool(SceneViewportTool::Rotate),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::AlignView(ViewOrientation::NegX),
    )
    .unwrap();

    let snapshot = harness.runtime.editor_snapshot();
    let packet = harness.runtime.render_snapshot().expect("render packet");

    assert_eq!(
        snapshot.scene_viewport_settings.display_mode,
        DisplayMode::WireOnly
    );
    assert_eq!(
        snapshot.scene_viewport_settings.grid_mode,
        GridMode::VisibleAndSnap
    );
    assert_eq!(
        snapshot.scene_viewport_settings.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(
        snapshot.scene_viewport_settings.tool,
        SceneViewportTool::Rotate
    );
    assert_eq!(
        snapshot.scene_viewport_settings.view_orientation,
        ViewOrientation::NegX
    );
    assert_eq!(packet.overlays.display_mode, DisplayMode::WireOnly);
    assert_eq!(
        packet.overlays.grid.as_ref().map(|grid| grid.snap_enabled),
        Some(true)
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
}
