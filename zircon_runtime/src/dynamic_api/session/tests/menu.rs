use super::*;
use crate::scene::components::NodeKind;

fn menu_extract(world: &World, viewport_size: UVec2) -> Option<UiRenderExtract> {
    let mut text_measure_cache = UiTextMeasureCache::default();
    text_measure_cache.begin_frame();
    let extract = runtime_session_menu_extract(world, viewport_size, &mut text_measure_cache);
    text_measure_cache.finish_frame();
    extract
}

#[test]
fn runtime_session_menu_extract_builds_start_button_commands() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({
                "state": "start",
                "title": "Blood Moon",
                "subtitle": "Hold the clearing",
                "button": "Start Game"
            }),
        )
        .unwrap();

    let extract = menu_extract(&world, UVec2::new(640, 360)).unwrap();

    assert_eq!(extract.list.commands.len(), 6);
    assert!(extract
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Start Game")));
    assert!(extract
        .list
        .commands
        .iter()
        .filter(|command| command.kind == UiRenderCommandKind::Text)
        .all(|command| command.text_layout.is_some()));
}

#[test]
fn runtime_session_menu_extract_resolves_cjk_text_before_renderer() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({
                "state": "start",
                "title": "血月降临",
                "subtitle": "这段中文菜单说明必须在渲染前使用规范文本管线完成自动换行与字形产物投影，并且必须裁剪在副标题框内以避免覆盖开始按钮",
                "button": "开始游戏"
            }),
        )
        .unwrap();

    let extract = menu_extract(&world, UVec2::new(320, 360)).unwrap();
    let subtitle = extract
        .list
        .commands
        .iter()
        .find(|command| {
            command
                .text
                .as_deref()
                .is_some_and(|text| text.starts_with("这段中文"))
        })
        .expect("CJK subtitle command");
    let layout = subtitle
        .text_layout
        .as_ref()
        .expect("menu text must carry the canonical layout");

    assert!(
        layout.lines.len() > 1,
        "fixture must soft-wrap without newlines"
    );
    assert!(layout.rich_text_artifact.is_some());
    assert_eq!(subtitle.clip_frame, Some(subtitle.frame));
    assert!(layout.overflow_clipped);
}

#[test]
fn runtime_session_fallback_ui_menu_lookup_uses_the_dynamic_component_sparse_index() {
    let source = include_str!("../menu.rs");
    let start = source
        .find("fn collect_runtime_menu_entity(")
        .expect("menu component lookup");
    let end = source[start..]
        .find("\nfn menu_from_value(")
        .map(|offset| start + offset)
        .expect("menu component lookup end");
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
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({ "state": "start", "button": "Indexed Start" }),
        )
        .unwrap();

    let extract = menu_extract(&world, UVec2::new(640, 360)).unwrap();
    assert!(extract
        .list
        .commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Indexed Start")));
}

#[test]
fn runtime_session_fallback_ui_menu_hit_test_does_not_build_presentation_strings() {
    let source = include_str!("../menu.rs");
    let start = source
        .find("pub(super) fn runtime_session_menu_action_at(")
        .expect("menu hit-test entry");
    let end = source[start..]
        .find("\npub(super) fn write_runtime_menu_action(")
        .map(|offset| start + offset)
        .expect("menu hit-test entry end");
    let hit_test_source = &source[start..end];
    assert!(hit_test_source.contains("menu_action_from_value"));
    assert!(!hit_test_source.contains("collect_runtime_menu("));

    let parser_start = source
        .find("fn menu_action_from_value(")
        .expect("menu action parser");
    let parser_end = source[parser_start..]
        .find("\nfn menu_action_from_state(")
        .map(|offset| parser_start + offset)
        .expect("menu action parser end");
    let parser_source = &source[parser_start..parser_end];
    assert!(!parser_source.contains("menu_string"));
    assert!(!parser_source.contains("String"));
    assert!(!parser_source.contains("to_string"));

    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({
                "state": "game_over",
                "title": "unused title",
                "subtitle": "unused subtitle",
                "button": "unused button"
            }),
        )
        .unwrap();
    let layout = menu_layout(UVec2::new(640, 360));

    assert_eq!(
        runtime_session_menu_action_at(
            &world,
            UVec2::new(640, 360),
            Vec2::new(layout.button.x + 8.0, layout.button.y + 8.0),
        ),
        Some(RuntimeMenuAction::RetryGame)
    );
}

#[test]
fn runtime_session_menu_action_writes_start_control_state() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            GAMEPLAY_MENU_COMPONENT,
            serde_json::json!({ "state": "start" }),
        )
        .unwrap();
    let layout = menu_layout(UVec2::new(640, 360));

    let action = runtime_session_menu_action_at(
        &world,
        UVec2::new(640, 360),
        Vec2::new(layout.button.x + 8.0, layout.button.y + 8.0),
    )
    .unwrap();
    assert_eq!(action, RuntimeMenuAction::StartGame);

    assert!(write_runtime_menu_action(&mut world, action));
    assert_eq!(
        world
            .dynamic_component(entity, GAMEPLAY_CONTROL_COMPONENT)
            .and_then(serde_json::Value::as_str),
        Some(START_BUTTON_ACTION)
    );
}
