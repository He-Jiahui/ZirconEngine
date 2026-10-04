use crate::core::math::UVec2;
use crate::scene::World;
use crate::ui::surface::{resolve_text_layout_with_cache, UiTextLayoutRequest, UiTextMeasureCache};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextRenderMode, UiTextWrap,
};

pub(super) const HUD_COMPONENT_IDS: [&str; 2] = ["gameplay.hud_text", "vampire.hud_text"];
const HUD_TREE_ID: &str = "runtime.gameplay.hud";
const HUD_MARGIN: f32 = 16.0;
const HUD_MIN_WIDTH: f32 = 220.0;
const HUD_MAX_WIDTH: f32 = 420.0;
const HUD_LINE_HEIGHT: f32 = 19.0;
const HUD_PADDING_X: f32 = 12.0;
const HUD_PADDING_Y: f32 = 10.0;
const HUD_MIN_HEIGHT: f32 = 48.0;
const HUD_MAX_HEIGHT: f32 = 220.0;

pub(super) fn runtime_session_hud_extract(
    world: &World,
    viewport_size: UVec2,
    text_measure_cache: &mut UiTextMeasureCache,
) -> Option<UiRenderExtract> {
    let text = collect_hud_text(world)?;
    if is_vampire_combat_hud_text(&text) {
        return None;
    }
    Some(build_text_hud_extract(
        text,
        viewport_size,
        text_measure_cache,
    ))
}

fn build_text_hud_extract(
    text: String,
    viewport_size: UVec2,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiRenderExtract {
    let width = hud_width(viewport_size);
    let text_frame = UiFrame::new(
        HUD_MARGIN + HUD_PADDING_X,
        HUD_MARGIN + HUD_PADDING_Y,
        (width - HUD_PADDING_X * 2.0).max(1.0),
        (HUD_MAX_HEIGHT - HUD_PADDING_Y * 2.0).max(1.0),
    );
    let text_style = hud_text_style();
    // Fallback UI extraction owns this measure/arrange work. Rendering receives the
    // canonical layout and only applies the final panel clip.
    let text_layout = resolve_text_layout_with_cache(
        &UiTextLayoutRequest::new(&text, &text_style, text_frame, None),
        text_measure_cache,
    )
    .layout;
    let height = hud_height(text_layout.measured_height);
    let panel_frame = UiFrame::new(HUD_MARGIN, HUD_MARGIN, width, height);
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
                    style: text_style,
                    text_layout: Some(text_layout),
                    text: Some(text),
                    image: None,
                    opacity: 1.0,
                },
            ],
        },
        raster_scale: 1.0,
    }
}

fn hud_text_style() -> UiResolvedStyle {
    UiResolvedStyle {
        foreground_color: Some("#f8fbffff".to_string()),
        font_size: 15.0,
        line_height: HUD_LINE_HEIGHT,
        wrap: UiTextWrap::Word,
        text_render_mode: UiTextRenderMode::Auto,
        ..UiResolvedStyle::default()
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

fn hud_height(measured_text_height: f32) -> f32 {
    (measured_text_height + HUD_PADDING_Y * 2.0).clamp(HUD_MIN_HEIGHT, HUD_MAX_HEIGHT)
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
#[path = "tests/hud.rs"]
mod tests;
