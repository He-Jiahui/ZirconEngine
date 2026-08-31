use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiRgbaColor,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::{
    metadata::control_label,
    state::SelectionRenderState,
    style::{SelectionVisual, label_color},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn push_label(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    opacity: f32,
) {
    let Some(label) = control_label(metadata) else {
        return;
    };
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return;
    }
    commands.push(text_command(
        node_id,
        frame,
        clip,
        z,
        label,
        label_color(state, visual),
        visual.label_font_size,
        visual.label_line_height,
        state,
        opacity,
    ));
}

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    let mut value = if alpha == u8::MAX {
        format!("{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}")
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
    state: &SelectionRenderState,
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
fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    state: &SelectionRenderState,
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
