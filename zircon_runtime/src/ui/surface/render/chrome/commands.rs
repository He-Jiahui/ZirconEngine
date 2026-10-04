//! 将外壳的表面、分隔线、文字、图标投影成共享渲染协议；几何和层级由内容层传入。
//! 所有命令保留原节点身份及裁剪，文字布局由提取器统一补齐，不能据此增设命中区域。
use std::borrow::Cow;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiPainterFamily,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::{
    metadata::ChromeKind,
    metrics::ChromeMetrics,
    state::ChromeRenderState,
    style::{border_color, border_width, corner_radius, separator_color, surface_color},
};

pub(super) fn surface_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    metadata: &UiTemplateNodeMetadata,
    kind: ChromeKind,
    state: &ChromeRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index: z_index.saturating_add(1),
        style: UiResolvedStyle {
            background_color: Some(surface_color(metadata, kind, state).into_owned()),
            border_color: border_color(metadata, state).map(Cow::into_owned),
            border_width: border_width(metadata, kind),
            corner_radius: corner_radius(metadata, kind),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state()),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

pub(super) fn separator_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    metadata: &UiTemplateNodeMetadata,
    state: &ChromeRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(separator_color(metadata, state).into_owned()),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state()),
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
    state: &ChromeRenderState,
    metrics: ChromeMetrics,
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
            font_size: metrics.font_size,
            line_height: metrics.line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state()),
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
    state: &ChromeRenderState,
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
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state()),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
}
