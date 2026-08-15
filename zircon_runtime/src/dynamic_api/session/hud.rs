use crate::core::math::UVec2;
use crate::scene::World;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextRenderMode, UiTextWrap,
};

const HUD_COMPONENT_IDS: [&str; 2] = ["gameplay.hud_text", "vampire.hud_text"];
const HUD_TREE_ID: &str = "runtime.gameplay.hud";
const HUD_MARGIN: f32 = 16.0;
const HUD_MIN_WIDTH: f32 = 220.0;
const HUD_MAX_WIDTH: f32 = 420.0;
const HUD_LINE_HEIGHT: f32 = 19.0;
const HUD_PADDING_X: f32 = 12.0;
const HUD_PADDING_Y: f32 = 10.0;

pub(super) fn runtime_session_hud_extract(
    world: &World,
    viewport_size: UVec2,
) -> Option<UiRenderExtract> {
    let text = collect_hud_text(world)?;
    if is_vampire_combat_hud_text(&text) {
        return None;
    }
    Some(build_text_hud_extract(text, viewport_size))
}

fn build_text_hud_extract(text: String, viewport_size: UVec2) -> UiRenderExtract {
    let width = hud_width(viewport_size);
    let height = hud_height(&text);
    let panel_frame = UiFrame::new(HUD_MARGIN, HUD_MARGIN, width, height);
    let text_frame = UiFrame::new(
        HUD_MARGIN + HUD_PADDING_X,
        HUD_MARGIN + HUD_PADDING_Y,
        (width - HUD_PADDING_X * 2.0).max(1.0),
        (height - HUD_PADDING_Y * 2.0).max(1.0),
    );
    UiRenderExtract {
        tree_id: UiTreeId::new(HUD_TREE_ID),
        list: UiRenderList {
            commands: vec![
                UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: panel_frame,
                    clip_frame: None,
                    z_index: 100,
                    style: UiResolvedStyle {
                        background_color: Some("#05070cff".to_string()),
                        border_color: Some("#b7e1ffff".to_string()),
                        border_width: 1.0,
                        corner_radius: 6.0,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: None,
                    image: None,
                    opacity: 1.0,
                },
                UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Text,
                    frame: text_frame,
                    clip_frame: Some(panel_frame),
                    z_index: 101,
                    style: UiResolvedStyle {
                        foreground_color: Some("#f8fbffff".to_string()),
                        font_size: 15.0,
                        line_height: HUD_LINE_HEIGHT,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some(text),
                    image: None,
                    opacity: 1.0,
                },
            ],
        },
        raster_scale: 1.0,
    }
}

fn collect_hud_text(world: &World) -> Option<String> {
    let mut rows = Vec::new();
    let mut selected = None;
    for (component_priority, component_id) in HUD_COMPONENT_IDS.into_iter().enumerate() {
        world.dynamic_component_rows(component_id, &mut rows);
        let Some((entity, text)) = rows
            .iter()
            .find_map(|(entity, value)| hud_text_from_value(value).map(|text| (*entity, text)))
        else {
            continue;
        };
        let should_replace = match &selected {
            Some((selected_entity, selected_priority, _)) => {
                (entity, component_priority) < (*selected_entity, *selected_priority)
            }
            None => true,
        };
        if should_replace {
            selected = Some((entity, component_priority, text));
        }
    }
    selected.map(|(_, _, text)| text)
}

fn hud_text_from_value(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .or_else(|| value.get("text").and_then(serde_json::Value::as_str))
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn hud_width(viewport_size: UVec2) -> f32 {
    let available = viewport_size.x.saturating_sub(32) as f32;
    available.clamp(HUD_MIN_WIDTH, HUD_MAX_WIDTH)
}

fn hud_height(text: &str) -> f32 {
    let line_count = text.lines().count().max(1) as f32;
    (line_count * HUD_LINE_HEIGHT + 24.0).clamp(48.0, 220.0)
}

fn is_vampire_combat_hud_text(text: &str) -> bool {
    let mut has_hp = false;
    let mut has_xp = false;
    let mut has_weapons = false;

    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let mut previous = None;
        for token in line
            .split_whitespace()
            .map(|token| token.trim_end_matches(':'))
        {
            match previous {
                Some("HP") => has_hp |= parse_f32_pair(token).is_some(),
                Some("XP") => has_xp |= parse_i64_pair(token).is_some(),
                Some("Orbit" | "Lance" | "Pulse") => has_weapons |= token.parse::<i64>().is_ok(),
                _ => {}
            }
            previous = Some(token);
        }
    }

    has_hp && has_xp && has_weapons
}

fn parse_i64_pair(value: &str) -> Option<(i64, i64)> {
    let (left, right) = value.split_once('/')?;
    Some((left.parse().ok()?, right.parse().ok()?))
}

fn parse_f32_pair(value: &str) -> Option<(f32, f32)> {
    let (left, right) = value.split_once('/')?;
    Some((left.parse().ok()?, right.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::components::NodeKind;

    #[test]
    fn runtime_session_hud_extract_reads_text_component() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_dynamic_component(
                entity,
                "gameplay.hud_text",
                serde_json::json!("Lv 2\nBuff: Haste"),
            )
            .unwrap();

        let extract = runtime_session_hud_extract(&world, UVec2::new(800, 600)).unwrap();
        let panel = extract.list.commands.first().unwrap();
        let text = extract.list.commands.get(1).unwrap();
        assert_eq!(text.text.as_deref(), Some("Lv 2\nBuff: Haste"));
        assert!(panel.frame.width >= HUD_MIN_WIDTH);
        assert!(text.frame.x > panel.frame.x);
        assert!(text.frame.y > panel.frame.y);
    }

    #[test]
    fn runtime_session_fallback_ui_hud_lookup_uses_the_dynamic_component_sparse_index() {
        let source = include_str!("hud.rs");
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
            world.spawn_node(NodeKind::Empty);
        }
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_dynamic_component(
                entity,
                "gameplay.hud_text",
                serde_json::json!("Indexed HUD"),
            )
            .unwrap();

        let extract = runtime_session_hud_extract(&world, UVec2::new(800, 600)).unwrap();
        assert!(extract
            .list
            .commands
            .iter()
            .any(|command| command.text.as_deref() == Some("Indexed HUD")));
    }

    #[test]
    fn runtime_session_hud_extract_suppresses_vampire_combat_panel_text() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty);
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
            runtime_session_hud_extract(&world, UVec2::new(1280, 720)).is_none(),
            "vampire health must render through scene-following world HUD bars, not a screen-space panel"
        );
    }

    #[test]
    fn vampire_combat_hud_detection_streams_tokens_without_collecting() {
        let source = include_str!("hud.rs");
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
}
