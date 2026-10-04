use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::{UiPainterFamily, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::super::resolve::resolve_style;
use super::{
    state::ButtonRenderState,
    style::{background_color, border_color, ButtonVisual},
};

pub(super) fn surface_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    state: &ButtonRenderState,
    visual: &ButtonVisual,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index: z_index.saturating_add(1),
        style: UiResolvedStyle {
            background_color: Some(css_color(background_color(state, visual))),
            border_color: Some(css_color(border_color(state, visual))),
            border_width: visual.border_width,
            corner_radius: if state.family() == UiPainterFamily::IconButton {
                visual.icon_button_radius
            } else {
                visual.button_radius
            },
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family(), state.visual_state()),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

pub(super) fn text_command(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    state: &ButtonRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            // The label inherits owner typography, but the surface owns its decoration.
            background_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            foreground_color: Some(css_color(foreground)),
            font_size,
            line_height,
            ..resolve_style(Some(metadata))
        }
        .with_painter_state(state.family(), state.visual_state()),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

pub(super) fn icon_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: String,
    foreground: UiRgbaColor,
    state: &ButtonRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(foreground)),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family(), state.visual_state()),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
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
