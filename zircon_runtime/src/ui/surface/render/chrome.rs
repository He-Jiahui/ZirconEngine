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

const SURFACE: &str = "#15191d";
const SURFACE_RAISED: &str = "#1b2226";
const SURFACE_INSET: &str = "#101418";
const SURFACE_VIEWPORT: &str = "#0b1115";
const SURFACE_STATUS: &str = "#11171b";
const SURFACE_HOVER: &str = "#202a2f";
const SURFACE_PRESSED: &str = "#14333b";
const SURFACE_SELECTED: &str = "#0f3b43";
const SURFACE_OPEN: &str = "#132e35";
const SURFACE_LOADING: &str = "#20262a";
const SURFACE_DISABLED: &str = "#252c31";
const BORDER: &str = "#2b343a";
const BORDER_MUTED: &str = "#242c31";
const BORDER_ACTIVE: &str = "#35c7d0";
const TEXT: &str = "#c6d2d7";
const TEXT_MUTED: &str = "#87939a";
const TEXT_DISABLED: &str = "#59656c";
const ICON: &str = "#9fb0b7";
const ACCENT: &str = "#35c7d0";
const FONT_SIZE: f32 = 11.5;
const LINE_HEIGHT: f32 = FONT_SIZE * 1.2;
const TEXT_INSET_X: f32 = 10.0;
const TEXT_INSET_Y: f32 = 7.0;
const ICON_SIZE: f32 = 16.0;
const ICON_GAP: f32 = 6.0;
const SEPARATOR_THICKNESS: f32 = 1.0;

pub(super) fn chrome_suppresses_owner_surface(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| chrome_kind(metadata).is_some())
}

pub(super) fn chrome_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| chrome_kind(metadata).is_some())
}

pub(super) fn chrome_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(|metadata| chrome_kind(metadata).is_some())
}

pub(super) fn chrome_render_commands(
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
    let Some(kind) = chrome_kind(metadata) else {
        return Vec::new();
    };
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = ChromeRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, metadata, kind, &state, opacity,
    )];

    if let Some(edge) = separator_edge(metadata, kind) {
        commands.push(separator_command(
            node_id,
            separator_frame(frame, edge),
            clip_frame,
            z_index.saturating_add(1),
            metadata,
            &state,
            opacity,
        ));
    }

    let label = chrome_label(metadata);
    let icon = chrome_icon(metadata);
    let has_icon = icon.is_some();
    if let Some(icon) = icon {
        let icon_size = number_attribute(metadata, "icon_size").unwrap_or(ICON_SIZE);
        commands.push(icon_command(
            node_id,
            icon_frame(frame, label.is_some(), icon_size),
            clip_frame,
            z_index.saturating_add(2),
            icon,
            icon_color(metadata, &state),
            &state,
            opacity,
        ));
    }
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            text_frame(frame, has_icon),
            clip_frame,
            z_index.saturating_add(2),
            label,
            text_color(metadata, &state),
            &state,
            opacity,
        ));
    }

    commands
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChromeKind {
    Shell,
    ActivityRail,
    Toolbar,
    StatusBar,
    Panel,
    Viewport,
}

#[derive(Clone, Copy)]
struct ChromeRenderState {
    visual_state: UiPainterResolvedState,
}

impl ChromeRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let visual_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state()
                .resolved_state_for_family(UiPainterFamily::Chrome);
        Self { visual_state }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn active(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Focused
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Selected
                | UiPainterResolvedState::Checked
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }

    fn selected_surface_active(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Open
                | UiPainterResolvedState::Selected
                | UiPainterResolvedState::Checked
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn chrome_kind(metadata: &UiTemplateNodeMetadata) -> Option<ChromeKind> {
    match metadata.component.as_str() {
        "WorkbenchShell" | "Shell" | "WorkbenchWindow" => Some(ChromeKind::Shell),
        "ActivityRail" | "ActivityRailPanel" => Some(ChromeKind::ActivityRail),
        "TopToolbar" | "Toolbar" | "MenuBar" | "WorkbenchMenuBar" => Some(ChromeKind::Toolbar),
        "StatusBar" | "BottomStatusBar" => Some(ChromeKind::StatusBar),
        "SceneTreePanel" | "InspectorPanel" | "Panel" | "DockPanel" | "ToolWindowStack" => {
            Some(ChromeKind::Panel)
        }
        "ViewportPanel" | "Viewport" | "SceneViewport" | "DocumentViewport" => {
            Some(ChromeKind::Viewport)
        }
        _ => match control_id(metadata) {
            Some(id) if id.contains("ActivityRail") => Some(ChromeKind::ActivityRail),
            Some(id) if id.contains("Toolbar") || id.contains("MenuBar") => {
                Some(ChromeKind::Toolbar)
            }
            Some(id) if id.contains("StatusBar") => Some(ChromeKind::StatusBar),
            Some(id) if id.contains("Viewport") => Some(ChromeKind::Viewport),
            Some(id) if id.contains("Panel") || id.contains("Dock") => Some(ChromeKind::Panel),
            Some(id) if id.contains("WorkbenchShell") || id.contains("WorkbenchWindow") => {
                Some(ChromeKind::Shell)
            }
            _ => None,
        },
    }
}

fn surface_command(
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
            background_color: Some(surface_color(metadata, kind, state).to_string()),
            border_color: border_color(metadata, state).map(str::to_string),
            border_width: border_width(metadata, kind),
            corner_radius: corner_radius(metadata, kind),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

fn separator_command(
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
            background_color: Some(
                color_attribute(metadata, "separator_color")
                    .unwrap_or_else(|| {
                        if state.active() {
                            BORDER_ACTIVE
                        } else {
                            BORDER_MUTED
                        }
                    })
                    .to_string(),
            ),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    state: &ChromeRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(foreground.to_string()),
            font_size: FONT_SIZE,
            line_height: LINE_HEIGHT,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

fn icon_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: String,
    foreground: &str,
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
            foreground_color: Some(foreground.to_string()),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(UiPainterFamily::Chrome, state.visual_state),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
}

fn surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    kind: ChromeKind,
    state: &ChromeRenderState,
) -> &'a str {
    if state.visual_state == UiPainterResolvedState::Disabled {
        SURFACE_DISABLED
    } else if state.visual_state == UiPainterResolvedState::Loading {
        SURFACE_LOADING
    } else if state.visual_state == UiPainterResolvedState::Pressed {
        color_attribute(metadata, "pressed_background_color").unwrap_or(SURFACE_PRESSED)
    } else if state.visual_state == UiPainterResolvedState::Open {
        color_attribute(metadata, "open_background_color").unwrap_or(SURFACE_OPEN)
    } else if state.visual_state == UiPainterResolvedState::Hovered {
        color_attribute(metadata, "hover_background_color").unwrap_or(SURFACE_HOVER)
    } else if state.selected_surface_active() {
        color_attribute(metadata, "selected_background_color").unwrap_or(SURFACE_SELECTED)
    } else {
        color_attribute(metadata, "background_color").unwrap_or_else(|| default_surface(kind))
    }
}

fn default_surface(kind: ChromeKind) -> &'static str {
    match kind {
        ChromeKind::Shell => SURFACE_INSET,
        ChromeKind::ActivityRail | ChromeKind::Toolbar => SURFACE_RAISED,
        ChromeKind::StatusBar => SURFACE_STATUS,
        ChromeKind::Panel => SURFACE,
        ChromeKind::Viewport => SURFACE_VIEWPORT,
    }
}

fn border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ChromeRenderState,
) -> Option<&'a str> {
    if state.unavailable() {
        Some(BORDER_MUTED)
    } else if state.active() {
        Some(color_attribute(metadata, "focus_border_color").unwrap_or(BORDER_ACTIVE))
    } else {
        color_attribute(metadata, "border_color").or(Some(BORDER))
    }
}

fn text_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &ChromeRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.active() {
        color_attribute(metadata, "active_foreground_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .unwrap_or(TEXT)
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TEXT_MUTED)
    }
}

fn icon_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &ChromeRenderState) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.active() {
        color_attribute(metadata, "active_icon_color")
            .or_else(|| color_attribute(metadata, "icon_color"))
            .unwrap_or(ACCENT)
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .unwrap_or(ICON)
    }
}

fn border_width(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> f32 {
    number_attribute(metadata, "border_width").unwrap_or_else(|| match kind {
        ChromeKind::Viewport => 0.0,
        _ => 1.0,
    })
}

fn corner_radius(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or_else(|| match kind {
            ChromeKind::Shell | ChromeKind::Toolbar | ChromeKind::StatusBar => 0.0,
            _ => 4.0,
        })
}

fn separator_edge(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> Option<SeparatorEdge> {
    string_attribute(metadata, "separator_edge")
        .and_then(parse_separator_edge)
        .or_else(|| match kind {
            ChromeKind::Toolbar => Some(SeparatorEdge::Bottom),
            ChromeKind::ActivityRail => Some(SeparatorEdge::Right),
            ChromeKind::StatusBar => Some(SeparatorEdge::Top),
            _ => None,
        })
}

#[derive(Clone, Copy)]
enum SeparatorEdge {
    Top,
    Right,
    Bottom,
    Left,
}

fn parse_separator_edge(value: &str) -> Option<SeparatorEdge> {
    match value.trim().to_ascii_lowercase().as_str() {
        "top" => Some(SeparatorEdge::Top),
        "right" => Some(SeparatorEdge::Right),
        "bottom" => Some(SeparatorEdge::Bottom),
        "left" => Some(SeparatorEdge::Left),
        "none" | "false" => None,
        _ => None,
    }
}

fn separator_frame(frame: UiFrame, edge: SeparatorEdge) -> UiFrame {
    match edge {
        SeparatorEdge::Top => UiFrame::new(frame.x, frame.y, frame.width, SEPARATOR_THICKNESS),
        SeparatorEdge::Right => UiFrame::new(
            frame.x + (frame.width - SEPARATOR_THICKNESS).max(0.0),
            frame.y,
            SEPARATOR_THICKNESS,
            frame.height,
        ),
        SeparatorEdge::Bottom => UiFrame::new(
            frame.x,
            frame.y + (frame.height - SEPARATOR_THICKNESS).max(0.0),
            frame.width,
            SEPARATOR_THICKNESS,
        ),
        SeparatorEdge::Left => UiFrame::new(frame.x, frame.y, SEPARATOR_THICKNESS, frame.height),
    }
}

fn text_frame(frame: UiFrame, has_icon: bool) -> UiFrame {
    let icon_offset = if has_icon { ICON_SIZE + ICON_GAP } else { 0.0 };
    UiFrame::new(
        frame.x + TEXT_INSET_X + icon_offset,
        frame.y + TEXT_INSET_Y,
        (frame.width - TEXT_INSET_X * 2.0 - icon_offset).max(1.0),
        (frame.height - TEXT_INSET_Y * 2.0).max(LINE_HEIGHT),
    )
}

fn icon_frame(frame: UiFrame, label_follows: bool, icon_size: f32) -> UiFrame {
    let x = if label_follows {
        frame.x + TEXT_INSET_X
    } else {
        frame.x + (frame.width - icon_size) * 0.5
    };
    UiFrame::new(
        x,
        frame.y + (frame.height - icon_size) * 0.5,
        icon_size,
        icon_size,
    )
}

fn chrome_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "title", "text", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn chrome_icon(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "icon")
        .or_else(|| string_attribute(metadata, "leading_icon"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn control_id(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    metadata.control_id.as_deref()
}

fn color_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(Value::as_str)
        .filter(|color| !color.trim().is_empty())
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
}

fn value_as_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Integer(value) => Some(*value as f32),
        Value::Float(value) if value.is_finite() => Some(*value as f32),
        _ => None,
    }
}
