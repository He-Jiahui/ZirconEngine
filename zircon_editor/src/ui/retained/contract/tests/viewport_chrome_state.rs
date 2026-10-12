use super::*;
use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
use crate::core::editor_message::PlayStateKind;
use crate::ui::retained_host::callback_dispatch::BuiltinViewportToolbarTemplateBridge;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn actual_viewport_values_and_command_play_admission_project_without_mutating_cached_source() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    let cached = bridge
        .paint_nodes_for_size(UiSize::new(640.0, 28.0))
        .unwrap();
    let mut viewport = SceneViewportChromeData {
        toolbar_template_nodes: cached.clone(),
        toolbar_surface_key: "document:first".into(),
        mode: "Select".into(),
        transform_space: "Local".into(),
        ..Default::default()
    };
    let registry = EditorCommandRegistry::default_workbench();
    for state in [
        PlayStateKind::Edit,
        PlayStateKind::Playing,
        PlayStateKind::Edit,
    ] {
        let context = CommandEvalCtx::interactive()
            .with_project_open(true)
            .with_play_state(state);
        viewport.toolbar_enter_play_enabled = registry
            .command("runtime.play_mode.enter")
            .unwrap()
            .is_enabled(&context);
        viewport.toolbar_exit_play_enabled = registry
            .command("runtime.play_mode.exit")
            .unwrap()
            .is_enabled(&context);
        viewport.toolbar_is_playing = state == PlayStateKind::Playing;
        let nodes = live_toolbar_nodes(&viewport, None);
        let enter = nodes
            .iter()
            .find(|node| node.control_id.as_str() == "document:first::EnterPlayMode")
            .unwrap();
        let stop = nodes
            .iter()
            .find(|node| node.control_id.as_str() == "document:first::ExitPlayMode")
            .unwrap();
        assert_eq!(enter.disabled, state != PlayStateKind::Edit);
        assert_eq!(stop.disabled, state != PlayStateKind::Playing);
        assert_eq!(stop.checked, state == PlayStateKind::Playing);
    }
    viewport.mode = "Transform.Move".into();
    viewport.transform_space = "Global".into();
    viewport.pivot_mode = "Origin".into();
    viewport.display_mode = "WireOverlay".into();
    viewport.grid_mode = "Hidden".into();
    let nodes = live_toolbar_nodes(&viewport, None);
    for (id, value) in [
        ("ActivateSceneMode", "Transform.Move"),
        ("SetTransformSpace", "Global"),
        ("SetPivotMode", "Origin"),
        ("SetDisplayMode", "WireOverlay"),
        ("SetGridMode", "Hidden"),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id.as_str() == format!("document:first::{id}"))
            .unwrap();
        assert!(node.selected);
        assert_eq!(node.value_text.as_str(), value);
        let source = cached
            .iter()
            .find(|node| node.control_id.as_str() == id)
            .unwrap();
        assert_eq!(node.icon_name, source.icon_name);
        assert_eq!(node.frame.x, source.frame.x);
        assert_eq!(node.source_node_id, source.source_node_id);
    }
    assert_eq!(
        cached
            .iter()
            .find(|node| node.control_id.as_str() == "ActivateSceneMode")
            .unwrap()
            .control_id
            .as_str(),
        "ActivateSceneMode"
    );
}
