use crate::core::math::{UVec2, Vec2};
use crate::scene::World;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};

pub(super) const GAMEPLAY_MENU_COMPONENT: &str = "gameplay.menu_state";
const GAMEPLAY_CONTROL_COMPONENT: &str = "gameplay.control_state";
const MENU_TREE_ID: &str = "runtime.gameplay.menu";
const START_BUTTON_ACTION: &str = "start_game";
const RETRY_BUTTON_ACTION: &str = "retry_game";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeMenuAction {
    StartGame,
    RetryGame,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeMenu {
    state: RuntimeMenuState,
    title: String,
    subtitle: String,
    button_label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeMenuState {
    Start,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MenuLayout {
    overlay: UiFrame,
    panel: UiFrame,
    title: UiFrame,
    subtitle: UiFrame,
    button: UiFrame,
    button_text: UiFrame,
}

pub(super) fn runtime_session_menu_extract(
    world: &World,
    viewport_size: UVec2,
) -> Option<UiRenderExtract> {
    let menu = collect_runtime_menu(world)?;
    let layout = menu_layout(viewport_size);
    Some(build_menu_extract(&menu, layout))
}

pub(super) fn runtime_session_menu_action_at(
    world: &World,
    viewport_size: UVec2,
    cursor: Vec2,
) -> Option<RuntimeMenuAction> {
    let (_, value) = collect_runtime_menu_entry(world)?;
    let action = menu_action_from_value(value)?;
    let layout = menu_layout(viewport_size);
    layout.button.contains(cursor).then_some(action)
}

pub(super) fn write_runtime_menu_action(world: &mut World, action: RuntimeMenuAction) -> bool {
    let Some(entity) = collect_runtime_menu_entity(world) else {
        return false;
    };
    let value = match action {
        RuntimeMenuAction::StartGame => START_BUTTON_ACTION,
        RuntimeMenuAction::RetryGame => RETRY_BUTTON_ACTION,
    };
    world
        .set_dynamic_component(entity, GAMEPLAY_CONTROL_COMPONENT, serde_json::json!(value))
        .unwrap_or(false)
}

fn collect_runtime_menu(world: &World) -> Option<RuntimeMenu> {
    let (_, value) = collect_runtime_menu_entry(world)?;
    menu_from_value(value)
}

fn collect_runtime_menu_entity(world: &World) -> Option<u64> {
    collect_runtime_menu_entry(world).map(|(entity, _)| entity)
}

fn collect_runtime_menu_entry(world: &World) -> Option<(u64, &serde_json::Value)> {
    let mut rows = Vec::new();
    world.dynamic_component_rows(GAMEPLAY_MENU_COMPONENT, &mut rows);
    rows.into_iter().next()
}

fn menu_state_from_value(value: &serde_json::Value) -> Option<RuntimeMenuState> {
    match value.get("state").and_then(serde_json::Value::as_str)? {
        "start" => Some(RuntimeMenuState::Start),
        "game_over" => Some(RuntimeMenuState::GameOver),
        _ => None,
    }
}

fn menu_action_from_value(value: &serde_json::Value) -> Option<RuntimeMenuAction> {
    menu_state_from_value(value).map(menu_action_from_state)
}

fn menu_action_from_state(state: RuntimeMenuState) -> RuntimeMenuAction {
    match state {
        RuntimeMenuState::Start => RuntimeMenuAction::StartGame,
        RuntimeMenuState::GameOver => RuntimeMenuAction::RetryGame,
    }
}

fn menu_from_value(value: &serde_json::Value) -> Option<RuntimeMenu> {
    let state = menu_state_from_value(value)?;
    let default_title = match state {
        RuntimeMenuState::Start => "Vampire Roguelite",
        RuntimeMenuState::GameOver => "Game Over",
    };
    let default_subtitle = match state {
        RuntimeMenuState::Start => "WASD movement. Blood Bolt attacks automatically.",
        RuntimeMenuState::GameOver => "The hunt ended. Return to the clearing and try again.",
    };
    let default_button = match state {
        RuntimeMenuState::Start => "Start Game",
        RuntimeMenuState::GameOver => "Retry",
    };
    let title = menu_string(value, "title", default_title);
    let subtitle = menu_string(value, "subtitle", default_subtitle);
    let button_label = menu_string(value, "button", default_button);
    Some(RuntimeMenu {
        state,
        title,
        subtitle,
        button_label,
    })
}

fn menu_string(value: &serde_json::Value, key: &str, fallback: &str) -> String {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn menu_layout(viewport_size: UVec2) -> MenuLayout {
    let width = viewport_size.x.max(1) as f32;
    let height = viewport_size.y.max(1) as f32;
    let panel_width = width.clamp(320.0, 520.0);
    let panel_height = 228.0_f32.min((height - 32.0).max(180.0));
    let panel_x = (width - panel_width) * 0.5;
    let panel_y = (height - panel_height) * 0.5;
    let title_y = panel_y + 28.0;
    let subtitle_y = title_y + 54.0;
    let button_width = (panel_width - 96.0).clamp(180.0, 280.0);
    let button_height = 48.0;
    let button_x = panel_x + (panel_width - button_width) * 0.5;
    let button_y = panel_y + panel_height - 74.0;

    MenuLayout {
        overlay: UiFrame::new(0.0, 0.0, width, height),
        panel: UiFrame::new(panel_x, panel_y, panel_width, panel_height),
        title: UiFrame::new(panel_x + 24.0, title_y, panel_width - 48.0, 44.0),
        subtitle: UiFrame::new(panel_x + 34.0, subtitle_y, panel_width - 68.0, 54.0),
        button: UiFrame::new(button_x, button_y, button_width, button_height),
        button_text: UiFrame::new(button_x + 12.0, button_y + 13.0, button_width - 24.0, 24.0),
    }
}

fn build_menu_extract(menu: &RuntimeMenu, layout: MenuLayout) -> UiRenderExtract {
    let accent = match menu.state {
        RuntimeMenuState::Start => "#d43a4bff",
        RuntimeMenuState::GameOver => "#6b1d28ff",
    };
    UiRenderExtract {
        tree_id: UiTreeId::new(MENU_TREE_ID),
        list: UiRenderList {
            commands: vec![
                quad_command(10, layout.overlay, 200, "#020507b8", None, 0.0, 1.0),
                quad_command(
                    11,
                    layout.panel,
                    201,
                    "#071011f2",
                    Some("#d7f7d7dd"),
                    1.0,
                    8.0,
                ),
                text_command(
                    12,
                    layout.title,
                    202,
                    &menu.title,
                    30.0,
                    "#f5fff1ff",
                    UiTextAlign::Center,
                ),
                text_command(
                    13,
                    layout.subtitle,
                    202,
                    &menu.subtitle,
                    16.0,
                    "#c7ddcaff",
                    UiTextAlign::Center,
                ),
                quad_command(14, layout.button, 203, accent, Some("#ffe4e8ff"), 1.0, 6.0),
                text_command(
                    15,
                    layout.button_text,
                    204,
                    &menu.button_label,
                    18.0,
                    "#fff8f8ff",
                    UiTextAlign::Center,
                ),
            ],
        },
        raster_scale: 1.0,
    }
}

fn quad_command(
    node_id: u64,
    frame: UiFrame,
    z_index: i32,
    background: &str,
    border: Option<&str>,
    border_width: f32,
    corner_radius: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame: None,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.to_string()),
            border_color: border.map(str::to_string),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

fn text_command(
    node_id: u64,
    frame: UiFrame,
    z_index: i32,
    text: &str,
    font_size: f32,
    color: &str,
    align: UiTextAlign,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: None,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(color.to_string()),
            font_size,
            line_height: font_size + 5.0,
            text_align: align,
            wrap: UiTextWrap::Word,
            text_render_mode: UiTextRenderMode::Auto,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}

trait MenuFrameHitTest {
    fn contains(&self, point: Vec2) -> bool;
}

impl MenuFrameHitTest for UiFrame {
    fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::components::NodeKind;

    #[test]
    fn runtime_session_menu_extract_builds_start_button_commands() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty);
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

        let extract = runtime_session_menu_extract(&world, UVec2::new(640, 360)).unwrap();

        assert_eq!(extract.list.commands.len(), 6);
        assert!(extract
            .list
            .commands
            .iter()
            .any(|command| command.text.as_deref() == Some("Start Game")));
    }

    #[test]
    fn runtime_session_fallback_ui_menu_lookup_uses_the_dynamic_component_sparse_index() {
        let source = include_str!("menu.rs");
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
            world.spawn_node(NodeKind::Empty);
        }
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_dynamic_component(
                entity,
                GAMEPLAY_MENU_COMPONENT,
                serde_json::json!({ "state": "start", "button": "Indexed Start" }),
            )
            .unwrap();

        let extract = runtime_session_menu_extract(&world, UVec2::new(640, 360)).unwrap();
        assert!(extract
            .list
            .commands
            .iter()
            .any(|command| command.text.as_deref() == Some("Indexed Start")));
    }

    #[test]
    fn runtime_session_fallback_ui_menu_hit_test_does_not_build_presentation_strings() {
        let source = include_str!("menu.rs");
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
        let entity = world.spawn_node(NodeKind::Empty);
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
        let entity = world.spawn_node(NodeKind::Empty);
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
}
