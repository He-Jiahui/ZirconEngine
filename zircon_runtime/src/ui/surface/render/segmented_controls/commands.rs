use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiRgbaColor,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
};

use super::state::SegmentedRenderState;

fn css_color(color: UiRgbaColor) -> String {
    let [r, g, b, a] = color.to_u8();
    let mut value = if a == u8::MAX {
        format!("{r:02x}{g:02x}{b:02x}")
    } else {
        format!("{r:02x}{g:02x}{b:02x}{a:02x}")
    };
    value.insert(0, '#');
    value
}

#[allow(clippy::too_many_arguments)]
pub(super) fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
    border_width: f32,
    corner_radius: f32,
    state: &SegmentedRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family(), state.visual_state()),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    state: &SegmentedRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(foreground)),
            font_size,
            line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family(), state.visual_state()),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
