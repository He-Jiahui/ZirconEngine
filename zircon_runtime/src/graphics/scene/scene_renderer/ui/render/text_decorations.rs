use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiPaintElement, UiPaintPayload, UiRenderCommand, UiTextDecorations, UiTextPaintDecorationKind,
    UiTextRange, UiTextWritingMode,
};

use super::color::{parse_color, parse_hex_color};
use super::geometry::{push_border, push_rect};
use super::ScreenSpaceUiVertex;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextDecorations {
    pub(in crate::graphics::scene::scene_renderer::ui) underline: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) strikethrough: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) underline_color: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::ui) strikethrough_color: [f32; 4],
}

pub(super) fn resolve_text_decorations(
    decorations: &UiTextDecorations,
    fallback_color: [f32; 4],
    opacity: f32,
) -> ScreenSpaceUiTextDecorations {
    ScreenSpaceUiTextDecorations {
        underline: decorations.underline,
        strikethrough: decorations.strikethrough,
        underline_color: resolved_decoration_color(
            decorations.underline_color.as_deref(),
            fallback_color,
            opacity,
        ),
        strikethrough_color: resolved_decoration_color(
            decorations.strikethrough_color.as_deref(),
            fallback_color,
            opacity,
        ),
    }
}

pub(super) fn resolved_text_decoration_baseline(
    command: &UiRenderCommand,
    source_range: Option<UiTextRange>,
    writing_mode: UiTextWritingMode,
) -> Option<f32> {
    let layout = command.text_layout.as_ref()?;
    let line = source_range
        .and_then(|range| {
            layout.lines.iter().find(|line| {
                line.source_range.start <= range.start && range.end <= line.source_range.end
            })
        })
        .or_else(|| layout.lines.first())?;
    let baseline = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        line.frame.x + line.baseline
    } else {
        line.frame.y + line.baseline
    };
    baseline.is_finite().then_some(baseline)
}

pub(super) fn push_text_decoration_vertices(
    paint_elements: &[UiPaintElement],
    command_opacity: f32,
    viewport: UiFrame,
    vertices: &mut Vec<ScreenSpaceUiVertex>,
    before_text: bool,
) {
    for element in paint_elements {
        let UiPaintPayload::Text { text } = &element.payload else {
            continue;
        };
        for decoration in &text.decorations {
            let decoration_before_text = matches!(
                decoration.kind,
                UiTextPaintDecorationKind::Selection
                    | UiTextPaintDecorationKind::CompositionHighlight
                    | UiTextPaintDecorationKind::TableCellBackground
            );
            if decoration_before_text != before_text {
                continue;
            }
            let Some(frame) = viewport.intersection(decoration.frame) else {
                continue;
            };
            let color = parse_color(
                Some(decoration.color.as_str()),
                text_decoration_fallback_color(decoration.kind),
                command_opacity,
            )
            .unwrap_or_else(|| text_decoration_fallback_color(decoration.kind));
            if matches!(decoration.kind, UiTextPaintDecorationKind::TableCellBorder) {
                push_border(vertices, frame, decoration.thickness, color, viewport);
            } else {
                push_rect(vertices, frame, color, viewport);
            }
        }
    }
}

fn text_decoration_fallback_color(kind: UiTextPaintDecorationKind) -> [f32; 4] {
    match kind {
        UiTextPaintDecorationKind::Selection => [0.30, 0.54, 1.0, 0.40],
        UiTextPaintDecorationKind::CompositionHighlight => [0.30, 0.54, 1.0, 0.14],
        UiTextPaintDecorationKind::CompositionUnderline => [0.30, 0.54, 1.0, 1.0],
        UiTextPaintDecorationKind::Caret => [0.91, 0.93, 0.97, 1.0],
        UiTextPaintDecorationKind::Outline => [0.91, 0.93, 0.97, 1.0],
        UiTextPaintDecorationKind::TableCellBackground => [0.0, 0.0, 0.0, 0.0],
        UiTextPaintDecorationKind::TableCellBorder => [0.91, 0.93, 0.97, 1.0],
    }
}

fn resolved_decoration_color(authored: Option<&str>, fallback: [f32; 4], opacity: f32) -> [f32; 4] {
    authored
        .and_then(|color| parse_hex_color(color, opacity))
        .unwrap_or(fallback)
}

#[cfg(test)]
#[path = "tests/text_decorations.rs"]
mod tests;
