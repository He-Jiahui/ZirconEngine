use std::borrow::Cow;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
};

use super::state::FeedbackRenderState;

pub(super) fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: Cow<'_, str>,
    border: Option<Cow<'_, str>>,
    border_width: f32,
    corner_radius: f32,
    state: &FeedbackRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.into_owned()),
            border_color: border.map(Cow::into_owned),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

pub(super) fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: Cow<'_, str>,
    font_size: f32,
    line_height: f32,
    state: &FeedbackRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(foreground.into_owned()),
            font_size,
            line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
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
    foreground: Cow<'_, str>,
    state: &FeedbackRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(foreground.into_owned()),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
}
