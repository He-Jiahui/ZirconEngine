use super::*;
use crate::scene::components::NodeKind;

fn hud_extract(world: &World, viewport_size: UVec2) -> Option<UiRenderExtract> {
    let mut text_measure_cache = UiTextMeasureCache::default();
    text_measure_cache.begin_frame();
    let extract = runtime_session_hud_extract(world, viewport_size, &mut text_measure_cache);
    text_measure_cache.finish_frame();
    extract
}

#[test]
fn runtime_session_hud_extract_reads_text_component() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            "gameplay.hud_text",
            serde_json::json!("Lv 2\nBuff: Haste"),
        )
        .unwrap();

    let extract = hud_extract(&world, UVec2::new(800, 600)).unwrap();
    let panel = extract.list.commands.first().unwrap();
    let text = extract.list.commands.get(1).unwrap();
    assert_eq!(text.text.as_deref(), Some("Lv 2\nBuff: Haste"));
    assert!(text.text_layout.is_some());
    assert!(panel.frame.width >= HUD_MIN_WIDTH);
    assert!(text.frame.x > panel.frame.x);
    assert!(text.frame.y > panel.frame.y);
}

#[test]
fn runtime_session_hud_extract_resolves_soft_wrapped_cjk_before_renderer() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            "gameplay.hud_text",
            serde_json::json!("中文文本布局需要在渲染前完成并根据自动换行扩展信息面板高度"),
        )
        .unwrap();

    let extract = hud_extract(&world, UVec2::new(252, 160)).unwrap();
    let panel = extract.list.commands.first().expect("HUD panel command");
    let text = extract.list.commands.get(1).expect("HUD text command");
    let layout = text
        .text_layout
        .as_ref()
        .expect("HUD text must carry the canonical layout");

    assert!(
        layout.lines.len() > 1,
        "fixture must soft-wrap without newlines"
    );
    assert!(layout.rich_text_artifact.is_some());
    assert!(panel.frame.height > HUD_MIN_HEIGHT);
    assert_eq!(text.clip_frame, Some(panel.frame));
}

#[test]
fn runtime_session_fallback_ui_hud_lookup_uses_the_dynamic_component_sparse_index() {
    let source = include_str!("../hud.rs");
    let start = source
        .find("fn collect_hud_text(")
        .expect("HUD component lookup");
    let end = source[start..]
        .find("\nfn hud_text_from_value(")
        .map(|offset| start + offset)
        .expect("HUD component lookup end");
    let lookup_source = &source[start..end];
    assert!(lookup_source.contains("dynamic_component_rows"));
    assert!(!lookup_source.contains("node_records()"));

    let mut world = World::empty();
    for _ in 0..4_096 {
        world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
    }
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            "gameplay.hud_text",
            serde_json::json!("Indexed HUD"),
        )
        .unwrap();

    let extract = hud_extract(&world, UVec2::new(800, 600)).unwrap();
    assert!(extract
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Indexed HUD")));
}

#[test]
fn runtime_session_hud_extract_suppresses_vampire_combat_panel_text() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            "gameplay.hud_text",
            serde_json::json!(
                "Lv 3  XP 9/30  HP 80/120\nTime 01:10  Kills 7  Enemies 4\nWeapons Orbit 1 Lance 2 Pulse 0\nShield 18  Blood 6s  Haste 5s"
            ),
        )
        .unwrap();

    assert!(
        hud_extract(&world, UVec2::new(1280, 720)).is_none(),
        "vampire health must render through scene-following world HUD bars, not a screen-space panel"
    );
}

#[test]
fn vampire_combat_hud_detection_streams_tokens_without_collecting() {
    let source = include_str!("../hud.rs");
    let start = source
        .find("fn is_vampire_combat_hud_text(")
        .expect("vampire combat HUD detector");
    let end = source[start..]
        .find("#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("vampire combat HUD detector end");
    let detector_source = &source[start..end];

    assert!(
        !detector_source.contains("collect::<Vec<_>>()"),
        "per-frame HUD classification must stream borrowed tokens without a temporary Vec"
    );
}
