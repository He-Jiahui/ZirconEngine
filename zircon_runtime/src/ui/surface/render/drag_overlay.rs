use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

const PREVIEW_SURFACE: &str = "#153035";
const PREVIEW_SURFACE_BLOCKED: &str = "#482024";
const PREVIEW_BORDER: &str = "#35c7d0";
const PREVIEW_BORDER_BLOCKED: &str = "#ef7066";
const PREVIEW_TEXT: &str = "#cee0e2";
const INDICATOR_ALLOWED: &str = "#35c7d0";
const INDICATOR_BLOCKED: &str = "#ef7066";
const PREVIEW_RADIUS: f32 = 6.0;
const ICON_LEFT: f32 = 12.0;
const ICON_SIZE: f32 = 18.0;
const TEXT_LEFT_WITH_ICON: f32 = 38.0;
const TEXT_RIGHT_INSET: f32 = 12.0;
const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 14.4;
const INDICATOR_THICKNESS: f32 = 2.0;

pub(super) fn drag_overlay_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_drag_overlay)
}

pub(super) fn drag_overlay_suppresses_owner_image(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_drag_overlay)
}

pub(super) fn drag_overlay_suppresses_owner_surface(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_drag_overlay)
}

pub(super) fn drag_overlay_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    state_flags: &UiStateFlags,
    component_state: Option<&UiComponentState>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !is_drag_overlay(metadata) || !drag_overlay_open(metadata, component_state) {
        return Vec::new();
    }
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let drop_allowed = bool_attribute(metadata, "drop_allowed").unwrap_or(true);
    let state = DragOverlayRenderState::resolve(metadata, state_flags, component_state);
    let preview_frame = preview_frame(metadata, frame);
    let mut commands = vec![quad_command(
        node_id,
        preview_frame,
        clip_frame,
        z_index.saturating_add(1),
        if drop_allowed {
            PREVIEW_SURFACE
        } else {
            PREVIEW_SURFACE_BLOCKED
        },
        Some(if drop_allowed {
            PREVIEW_BORDER
        } else {
            PREVIEW_BORDER_BLOCKED
        }),
        1.0,
        PREVIEW_RADIUS,
        state.preview_state,
        opacity,
    )];

    let icon = payload_icon(metadata);
    if let Some(icon) = icon {
        commands.push(image_command(
            node_id,
            UiFrame::new(
                preview_frame.x + ICON_LEFT,
                preview_frame.y + (preview_frame.height - ICON_SIZE).max(0.0) * 0.5,
                ICON_SIZE,
                ICON_SIZE,
            ),
            clip_frame,
            z_index.saturating_add(2),
            icon,
            if drop_allowed {
                PREVIEW_BORDER
            } else {
                PREVIEW_BORDER_BLOCKED
            },
            state.preview_state,
            opacity,
        ));
    }

    if let Some(label) = preview_label(metadata) {
        let text_left = preview_frame.x
            + if icon.is_some() {
                TEXT_LEFT_WITH_ICON
            } else {
                ICON_LEFT
            };
        let text_width = (preview_frame.right() - TEXT_RIGHT_INSET - text_left).max(1.0);
        commands.push(text_command(
            node_id,
            UiFrame::new(
                text_left,
                preview_frame.y + (preview_frame.height - LINE_HEIGHT).max(0.0) * 0.5,
                text_width,
                LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(3),
            label,
            PREVIEW_TEXT,
            state.preview_state,
            opacity,
        ));
    }

    if let Some(indicator_frame) = indicator_frame(metadata) {
        commands.push(quad_command(
            node_id,
            indicator_frame,
            clip_frame,
            z_index.saturating_add(4),
            if drop_allowed {
                INDICATOR_ALLOWED
            } else {
                INDICATOR_BLOCKED
            },
            None,
            0.0,
            1.0,
            UiPainterResolvedState::DropHovered,
            opacity,
        ));
    }

    commands
}

#[derive(Clone, Copy)]
struct DragOverlayRenderState {
    preview_state: UiPainterResolvedState,
}

impl DragOverlayRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        Self {
            preview_state: painter_state.resolved_state_for_family(UiPainterFamily::Chrome),
        }
    }
}

fn is_drag_overlay(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "DragOverlay"
}

fn drag_overlay_open(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
) -> bool {
    bool_attribute(metadata, "open").unwrap_or(false)
        || bool_attribute(metadata, "dragging").unwrap_or(false)
        || component_state.is_some_and(|state| state.flags.dragging)
}

fn preview_frame(metadata: &UiTemplateNodeMetadata, fallback: UiFrame) -> UiFrame {
    let width = number_attribute(metadata, "preview_width")
        .unwrap_or(fallback.width)
        .max(1.0);
    let height = number_attribute(metadata, "preview_height")
        .unwrap_or(fallback.height)
        .max(1.0);
    match (
        number_attribute(metadata, "cursor_x"),
        number_attribute(metadata, "cursor_y"),
    ) {
        (Some(x), Some(y)) => UiFrame::new(
            x + number_attribute(metadata, "offset_x").unwrap_or(12.0),
            y + number_attribute(metadata, "offset_y").unwrap_or(12.0),
            width,
            height,
        ),
        _ => UiFrame::new(fallback.x, fallback.y, width, height),
    }
}

fn indicator_frame(metadata: &UiTemplateNodeMetadata) -> Option<UiFrame> {
    let edge = string_attribute(metadata, "drop_indicator_edge").unwrap_or("none");
    if edge == "none" {
        return None;
    }
    let x = number_attribute(metadata, "drop_target_x")?;
    let y = number_attribute(metadata, "drop_target_y")?;
    let width = number_attribute(metadata, "drop_target_width")
        .unwrap_or(0.0)
        .max(1.0);
    let height = number_attribute(metadata, "drop_target_height")
        .unwrap_or(0.0)
        .max(1.0);
    match edge {
        "top" => Some(UiFrame::new(x, y, width, INDICATOR_THICKNESS)),
        "bottom" => Some(UiFrame::new(
            x,
            y + (height - INDICATOR_THICKNESS).max(0.0),
            width,
            INDICATOR_THICKNESS,
        )),
        "left" => Some(UiFrame::new(x, y, INDICATOR_THICKNESS, height)),
        "right" => Some(UiFrame::new(
            x + (width - INDICATOR_THICKNESS).max(0.0),
            y,
            INDICATOR_THICKNESS,
            height,
        )),
        "inside" => Some(UiFrame::new(x, y, width, height)),
        _ => None,
    }
}

fn preview_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "payload_label")
        .filter(|value| !value.is_empty())
        .or_else(|| string_attribute(metadata, "text").filter(|value| !value.is_empty()))
        .or_else(|| {
            string_attribute(metadata, "payload_reference").filter(|value| !value.is_empty())
        })
        .map(ToOwned::to_owned)
}

fn payload_icon(metadata: &UiTemplateNodeMetadata) -> Option<&'static str> {
    match string_attribute(metadata, "payload_kind").unwrap_or("unknown") {
        "asset" => Some("package"),
        "scene-instance" => Some("box"),
        "object" => Some("cube"),
        _ => None,
    }
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background_color: &str,
    border_color: Option<&str>,
    border_width: f32,
    corner_radius: f32,
    painter_state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background_color.to_string()),
            border_color: border_color.map(ToOwned::to_owned),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default().with_painter_state(UiPainterFamily::Chrome, painter_state)
        },
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

fn image_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: &str,
    color: &str,
    painter_state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(color.to_string()),
            ..UiResolvedStyle::default().with_painter_state(UiPainterFamily::Chrome, painter_state)
        },
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon.to_string())),
        opacity,
    }
}

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    color: &str,
    painter_state: UiPainterResolvedState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(color.to_string()),
            font_size: FONT_SIZE,
            line_height: LINE_HEIGHT,
            ..UiResolvedStyle::default().with_painter_state(UiPainterFamily::Chrome, painter_state)
        },
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(|value| {
        value
            .as_float()
            .map(|value| value as f32)
            .or_else(|| value.as_integer().map(|value| value as f32))
    })
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}
