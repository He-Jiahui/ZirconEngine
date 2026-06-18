use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_inspector_row_geometry::INSPECTOR_ROW_TEXT_Y;
use super::super::template_inspector_row_glyphs::push_inspector_down_chevron;
use super::primitives::push_text;
use super::style::{disclosure_label_color, INSPECTOR_GLYPH_COLOR};

pub(super) fn push_disclosure_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let chevron = FrameRect {
        x: rect.x + 2.0,
        y: rect.y + (rect.height - 12.0).max(0.0) * 0.5,
        width: 12.0,
        height: 12.0,
    };
    push_inspector_down_chevron(
        commands,
        &chevron,
        clip,
        order,
        INSPECTOR_GLYPH_COLOR,
        opacity,
    );
    push_text(
        commands,
        FrameRect {
            x: chevron.x + chevron.width + 5.0,
            y: rect.y + INSPECTOR_ROW_TEXT_Y,
            width: (rect.width - chevron.width - 12.0).max(1.0),
            height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
        },
        clip,
        order + 1,
        node.text.trim(),
        disclosure_label_color(),
        opacity,
    );
}
