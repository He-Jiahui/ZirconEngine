use std::{borrow::Cow, sync::OnceLock};

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::EditorDesignTokens,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Debug)]
struct ChromePalette {
    surface: String,
    surface_raised: String,
    surface_inset: String,
    surface_viewport: String,
    surface_status: String,
    surface_hover: String,
    surface_pressed: String,
    surface_selected: String,
    surface_open: String,
    surface_loading: String,
    surface_disabled: String,
    border: String,
    border_muted: String,
    border_active: String,
    text: String,
    text_muted: String,
    text_disabled: String,
    icon: String,
    accent: String,
    border_width: f32,
    radius_small: f32,
}

#[derive(Clone, Copy)]
struct ChromeMetrics {
    text_inset_left: f32,
    text_inset_right: f32,
    text_inset_y: f32,
    icon_size: f32,
    icon_gap: f32,
    separator_thickness: f32,
    font_size: f32,
    line_height: f32,
}

impl ChromeMetrics {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let default_metrics = default_chrome_metrics();
        let default_line_height_ratio = default_metrics.line_height / default_metrics.font_size;
        let mut metrics = default_metrics;
        metrics.text_inset_left = metric_attribute(metadata, "layout_padding_left")
            .or_else(|| metric_attribute(metadata, "text_inset_left"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.text_inset_left);
        metrics.text_inset_right = metric_attribute(metadata, "layout_padding_right")
            .or_else(|| metric_attribute(metadata, "text_inset_right"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.text_inset_right);
        metrics.text_inset_y = metric_attribute(metadata, "layout_padding_vertical")
            .or_else(|| metric_attribute(metadata, "text_inset_y"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.text_inset_y);
        metrics.icon_size = metric_attribute(metadata, "layout_icon_size")
            .or_else(|| metric_attribute(metadata, "icon_size"))
            .filter(|value| *value > 0.0)
            .unwrap_or(metrics.icon_size);
        metrics.icon_gap = metric_attribute(metadata, "layout_spacing")
            .or_else(|| metric_attribute(metadata, "icon_gap"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.icon_gap);
        metrics.separator_thickness = metric_attribute(metadata, "separator_thickness")
            .or_else(|| metric_attribute(metadata, "border_width"))
            .filter(|value| *value > 0.0)
            .unwrap_or(metrics.separator_thickness);
        metrics.font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(metrics.font_size);
        metrics.line_height = metric_attribute(metadata, "line_height")
            .filter(|value| *value > 0.0)
            .or_else(|| {
                metric_attribute(metadata, "line_height_ratio")
                    .filter(|value| *value > 0.0)
                    .map(|ratio| metrics.font_size * ratio)
            })
            .unwrap_or(metrics.font_size * default_line_height_ratio);
        metrics
    }
}

fn default_chrome_metrics() -> ChromeMetrics {
    static METRICS: OnceLock<ChromeMetrics> = OnceLock::new();
    *METRICS.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        ChromeMetrics {
            text_inset_left: (density.gap_large - controls.border_width * 2.0).max(0.0),
            text_inset_right: (density.gap_large - controls.border_width * 2.0).max(0.0),
            text_inset_y: (density.gap_medium - controls.border_width).max(0.0),
            icon_size: (controls.dense_height - density.gap_large).max(controls.border_width),
            icon_gap: (density.gap_medium - controls.border_width * 2.0).max(0.0),
            separator_thickness: controls.border_width,
            font_size: typography.body_size,
            line_height: typography.body_size * typography.line_height,
        }
    })
}

fn chrome_palette() -> &'static ChromePalette {
    static PALETTE: OnceLock<ChromePalette> = OnceLock::new();
    PALETTE.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        ChromePalette {
            surface: css_color(colors.surface[1]),
            surface_raised: css_color(colors.surface[2]),
            surface_inset: css_color(colors.surface_recessed),
            surface_viewport: css_color(colors.surface_recessed),
            surface_status: css_color(colors.surface[0]),
            surface_hover: css_color(colors.surface_hover),
            surface_pressed: css_color(colors.surface[3]),
            surface_selected: css_color(colors.surface_selected),
            surface_open: css_color(colors.accent_soft),
            surface_loading: css_color(colors.surface[2]),
            surface_disabled: css_color(colors.surface_disabled),
            border: css_color(colors.border),
            border_muted: css_color(colors.separator_soft),
            border_active: css_color(colors.accent),
            text: css_color(colors.text_primary),
            text_muted: css_color(colors.text_secondary),
            text_disabled: css_color(colors.text_disabled),
            icon: css_color(colors.text_secondary),
            accent: css_color(colors.accent),
            border_width: controls.border_width,
            radius_small: controls.small_radius,
        }
    })
}

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
    let metrics = ChromeMetrics::resolve(metadata);
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, metadata, kind, &state, opacity,
    )];

    if let Some(edge) = separator_edge(metadata, kind) {
        commands.push(separator_command(
            node_id,
            separator_frame(frame, edge, metrics.separator_thickness),
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
        commands.push(icon_command(
            node_id,
            icon_frame(frame, label.is_some(), metrics),
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
            text_frame(frame, has_icon, metrics),
            clip_frame,
            z_index.saturating_add(2),
            label,
            text_color(metadata, &state),
            &state,
            metrics,
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
            background_color: Some(surface_color(metadata, kind, state).into_owned()),
            border_color: border_color(metadata, state).map(Cow::into_owned),
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
                    .map(Cow::Borrowed)
                    .unwrap_or_else(|| {
                        if state.active() {
                            Cow::Borrowed(&chrome_palette().border_active)
                        } else {
                            Cow::Borrowed(&chrome_palette().border_muted)
                        }
                    })
                    .into_owned(),
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
) -> Cow<'a, str> {
    if state.visual_state == UiPainterResolvedState::Disabled {
        Cow::Borrowed(&chrome_palette().surface_disabled)
    } else if state.visual_state == UiPainterResolvedState::Loading {
        Cow::Borrowed(&chrome_palette().surface_loading)
    } else if state.visual_state == UiPainterResolvedState::Pressed {
        color_attribute(metadata, "pressed_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().surface_pressed))
    } else if state.visual_state == UiPainterResolvedState::Open {
        color_attribute(metadata, "open_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().surface_open))
    } else if state.visual_state == UiPainterResolvedState::Hovered {
        color_attribute(metadata, "hover_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().surface_hover))
    } else if state.selected_surface_active() {
        color_attribute(metadata, "selected_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().surface_selected))
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| default_surface(kind))
    }
}

fn default_surface<'a>(kind: ChromeKind) -> Cow<'a, str> {
    let palette = chrome_palette();
    Cow::Borrowed(match kind {
        ChromeKind::Shell => &palette.surface_inset,
        ChromeKind::ActivityRail | ChromeKind::Toolbar => &palette.surface_raised,
        ChromeKind::StatusBar => &palette.surface_status,
        ChromeKind::Panel => &palette.surface,
        ChromeKind::Viewport => &palette.surface_viewport,
    })
}

fn border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ChromeRenderState,
) -> Option<Cow<'a, str>> {
    if state.unavailable() {
        Some(Cow::Borrowed(&chrome_palette().border_muted))
    } else if state.active() {
        Some(
            color_attribute(metadata, "focus_border_color")
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().border_active)),
        )
    } else {
        Some(
            color_attribute(metadata, "border_color")
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().border)),
        )
    }
}

fn text_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &ChromeRenderState) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&chrome_palette().text_disabled)
    } else if state.active() {
        color_attribute(metadata, "active_foreground_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().text))
    } else {
        color_attribute(metadata, "foreground_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().text_muted))
    }
}

fn icon_color<'a>(metadata: &'a UiTemplateNodeMetadata, state: &ChromeRenderState) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&chrome_palette().text_disabled)
    } else if state.active() {
        color_attribute(metadata, "active_icon_color")
            .or_else(|| color_attribute(metadata, "icon_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().accent))
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "foreground_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().icon))
    }
}

fn border_width(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> f32 {
    number_attribute(metadata, "border_width").unwrap_or_else(|| match kind {
        ChromeKind::Viewport => 0.0,
        _ => chrome_palette().border_width,
    })
}

fn corner_radius(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or_else(|| match kind {
            ChromeKind::Shell | ChromeKind::Toolbar | ChromeKind::StatusBar => 0.0,
            _ => chrome_palette().radius_small,
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
    let value = value.trim();
    if value.eq_ignore_ascii_case("top") {
        Some(SeparatorEdge::Top)
    } else if value.eq_ignore_ascii_case("right") {
        Some(SeparatorEdge::Right)
    } else if value.eq_ignore_ascii_case("bottom") {
        Some(SeparatorEdge::Bottom)
    } else if value.eq_ignore_ascii_case("left") {
        Some(SeparatorEdge::Left)
    } else {
        None
    }
}

fn separator_frame(frame: UiFrame, edge: SeparatorEdge, thickness: f32) -> UiFrame {
    match edge {
        SeparatorEdge::Top => UiFrame::new(frame.x, frame.y, frame.width, thickness),
        SeparatorEdge::Right => UiFrame::new(
            frame.x + (frame.width - thickness).max(0.0),
            frame.y,
            thickness,
            frame.height,
        ),
        SeparatorEdge::Bottom => UiFrame::new(
            frame.x,
            frame.y + (frame.height - thickness).max(0.0),
            frame.width,
            thickness,
        ),
        SeparatorEdge::Left => UiFrame::new(frame.x, frame.y, thickness, frame.height),
    }
}

fn text_frame(frame: UiFrame, has_icon: bool, metrics: ChromeMetrics) -> UiFrame {
    let icon_offset = if has_icon {
        metrics.icon_size + metrics.icon_gap
    } else {
        0.0
    };
    UiFrame::new(
        frame.x + metrics.text_inset_left + icon_offset,
        frame.y + metrics.text_inset_y,
        (frame.width - metrics.text_inset_left - metrics.text_inset_right - icon_offset).max(1.0),
        (frame.height - metrics.text_inset_y * 2.0).max(metrics.line_height),
    )
}

fn icon_frame(frame: UiFrame, label_follows: bool, metrics: ChromeMetrics) -> UiFrame {
    let x = if label_follows {
        frame.x + metrics.text_inset_left
    } else {
        frame.x + (frame.width - metrics.icon_size) * 0.5
    };
    UiFrame::new(
        x,
        frame.y + (frame.height - metrics.icon_size) * 0.5,
        metrics.icon_size,
        metrics.icon_size,
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
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    number_attribute(metadata, key).filter(|value| value.is_finite())
}

fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
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
