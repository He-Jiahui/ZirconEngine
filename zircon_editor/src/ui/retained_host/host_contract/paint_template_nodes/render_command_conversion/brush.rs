use zircon_runtime_interface::ui::surface::{UiBrushPayload, UiRenderResourceKey};

use super::style::parse_style_color;

pub(super) fn brush_fill_color(brush: &UiBrushPayload) -> Option<[u8; 4]> {
    match brush {
        UiBrushPayload::Solid(payload) => parse_style_color(Some(&payload.color)),
        UiBrushPayload::Rounded(payload) => parse_style_color(Some(&payload.color)),
        UiBrushPayload::Material(payload) => parse_style_color(payload.fallback_color.as_deref()),
        UiBrushPayload::Gradient(payload) => payload
            .stops
            .first()
            .and_then(|stop| parse_style_color(Some(&stop.color))),
        _ => None,
    }
}

pub(super) fn brush_border(brush: &UiBrushPayload) -> Option<(Option<[u8; 4]>, f32)> {
    match brush {
        UiBrushPayload::Border(payload) => {
            Some((parse_style_color(Some(&payload.color)), payload.width))
        }
        _ => None,
    }
}

pub(super) fn image_brush_resource(brush: &UiBrushPayload) -> Option<&UiRenderResourceKey> {
    match brush {
        UiBrushPayload::Image(payload) | UiBrushPayload::Box(payload) => Some(&payload.resource),
        UiBrushPayload::Vector(payload) => Some(&payload.resource),
        _ => None,
    }
}
