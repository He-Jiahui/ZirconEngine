use std::{collections::HashSet, sync::OnceLock};

use zircon_runtime_interface::ui::{
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::UiNodeId,
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiPainterState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::popup_position::{
    PopupPlacement, anchored_popup_frame, has_popup_position_metadata, popup_anchor_frame,
    popup_layout_bounds,
};

#[derive(Clone, Debug)]
struct PopupRowPalette {
    background: String,
    border: String,
    selected_background: String,
    hover_background: String,
    text: String,
    muted_text: String,
    accent_text: String,
    danger_text: String,
    row_min_height: f32,
    row_text_inset_x: f32,
    separator_inset: f32,
    option_row_gap: f32,
    selected_mark_width: f32,
    border_width: f32,
    min_frame_extent: f32,
    shadow: String,
    panel_radius: f32,
    row_radius: f32,
    font_size: f32,
    line_height: f32,
}

fn popup_row_palette() -> &'static PopupRowPalette {
    static PALETTE: OnceLock<PopupRowPalette> = OnceLock::new();
    PALETTE.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        PopupRowPalette {
            background: css_color(colors.popup),
            border: css_color(colors.border),
            selected_background: css_color(colors.surface_selected),
            hover_background: css_color(colors.surface_hover),
            text: css_color(colors.text_primary),
            muted_text: css_color(colors.text_disabled),
            accent_text: css_color(colors.accent),
            danger_text: css_color(colors.error),
            row_min_height: (controls.dense_height - density.gap_small).max(controls.border_width),
            row_text_inset_x: density.gap_medium,
            separator_inset: density.gap_medium,
            option_row_gap: density.gap_small,
            selected_mark_width: controls.border_width + density.gap_small * 0.5,
            border_width: controls.border_width,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
            shadow: css_color(colors.shadow),
            panel_radius: controls.panel_radius,
            row_radius: controls.small_radius,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
        }
    })
}

const POPUP_RENDER_Z_OFFSET: i32 = 100;
const POPUP_ATTRIBUTE_LINEAR_SCAN_LIMIT: usize = 8;
const POPUP_SHADOW_NEAR_OPACITY: f32 = 0.16;
const POPUP_SHADOW_FAR_OPACITY: f32 = 0.10;

pub(super) enum PopupAttributeIdSet<'a> {
    Empty,
    Single(&'a str),
    Linear(&'a [toml::Value]),
    Indexed(HashSet<&'a str>),
}

impl<'a> PopupAttributeIdSet<'a> {
    pub(super) fn new(value: Option<&'a toml::Value>) -> Self {
        match value {
            Some(toml::Value::String(value)) => Self::Single(value.as_str()),
            Some(toml::Value::Array(values)) => {
                let string_count = values.iter().filter_map(toml::Value::as_str).count();
                if string_count == 0 {
                    Self::Empty
                } else if string_count <= POPUP_ATTRIBUTE_LINEAR_SCAN_LIMIT {
                    Self::Linear(values.as_slice())
                } else {
                    let mut index = HashSet::with_capacity(string_count);
                    index.extend(values.iter().filter_map(toml::Value::as_str));
                    Self::Indexed(index)
                }
            }
            _ => Self::Empty,
        }
    }

    pub(super) fn contains(&self, value: &str) -> bool {
        match self {
            Self::Empty => false,
            Self::Single(entry) => *entry == value,
            Self::Linear(entries) => entries
                .iter()
                .filter_map(toml::Value::as_str)
                .any(|entry| entry == value),
            Self::Indexed(entries) => entries.contains(value),
        }
    }

    pub(super) fn contains_any(&self, first: &str, second: &str) -> bool {
        self.contains(first) || self.contains(second)
    }
}

pub(crate) fn popup_base_z(z_index: i32) -> i32 {
    z_index.saturating_add(POPUP_RENDER_Z_OFFSET)
}

pub(super) fn menu_row_height(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    row_count: usize,
) -> Option<f32> {
    let layout = popup_row_layout(metadata);
    popup_row_height_with_layout(frame, row_count, layout)
        .map(|height| height.max(popup_row_palette().row_min_height))
}

pub(super) fn popup_rows_height(
    metadata: &UiTemplateNodeMetadata,
    row_count: usize,
    row_height: f32,
) -> Option<f32> {
    if row_count == 0 || !row_height.is_finite() || row_height <= 0.0 {
        return None;
    }
    let layout = popup_row_layout(metadata);
    Some(
        layout.top
            + layout.bottom
            + row_height * row_count as f32
            + layout.spacing * row_count.saturating_sub(1) as f32,
    )
}

pub(super) fn popup_row_frame(
    metadata: &UiTemplateNodeMetadata,
    popup_frame: UiFrame,
    row_count: usize,
    row: usize,
) -> Option<UiFrame> {
    if row >= row_count {
        return None;
    }
    let layout = popup_row_layout(metadata);
    let row_height = popup_row_height_with_layout(popup_frame, row_count, layout)?;
    let width = popup_frame.width - layout.left - layout.right;
    if !width.is_finite() || width <= 0.0 {
        return None;
    }
    Some(UiFrame::new(
        popup_frame.x + layout.left,
        popup_frame.y + layout.top + row as f32 * (row_height + layout.spacing),
        width,
        row_height,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PopupRowLayout {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    spacing: f32,
}

fn popup_row_layout(metadata: &UiTemplateNodeMetadata) -> PopupRowLayout {
    PopupRowLayout {
        left: popup_layout_metric(metadata, "layout_padding_left", 0.0),
        right: popup_layout_metric(metadata, "layout_padding_right", 0.0),
        top: popup_layout_metric(metadata, "layout_padding_top", 0.0),
        bottom: popup_layout_metric(metadata, "layout_padding_bottom", 0.0),
        spacing: popup_layout_metric(metadata, "layout_spacing", 0.0),
    }
}

fn popup_row_height_with_layout(
    frame: UiFrame,
    row_count: usize,
    layout: PopupRowLayout,
) -> Option<f32> {
    if row_count == 0 || !frame.height.is_finite() || frame.height <= 0.0 {
        return None;
    }
    let fixed_height =
        layout.top + layout.bottom + layout.spacing * row_count.saturating_sub(1) as f32;
    let height = (frame.height - fixed_height) / row_count as f32;
    (height.is_finite() && height > 0.0).then_some(height)
}

fn popup_layout_metric(metadata: &UiTemplateNodeMetadata, property: &str, fallback: f32) -> f32 {
    metadata
        .attributes
        .get(property)
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or(fallback)
}

pub(super) fn option_popup_layout_bounds(
    control_frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> Option<UiFrame> {
    popup_layout_bounds(control_frame, clip_frame)
}

pub(super) fn option_popup_frame_within(
    metadata: &UiTemplateNodeMetadata,
    control_frame: UiFrame,
    anchor_frame: UiFrame,
    row_count: usize,
    bounds: Option<UiFrame>,
) -> Option<UiFrame> {
    if row_count == 0 {
        return None;
    }
    let row_height = option_row_height(metadata, control_frame, row_count);
    if metadata.component == "DropdownPopup" && !has_popup_position_metadata(metadata) {
        return Some(control_frame);
    }
    let anchor_frame = popup_anchor_frame(metadata, anchor_frame);
    anchored_popup_frame(
        metadata,
        anchor_frame,
        control_frame
            .width
            .max(popup_row_palette().min_frame_extent),
        popup_rows_height(metadata, row_count, row_height)?,
        bounds,
        PopupPlacement::BottomStart,
        popup_row_palette().option_row_gap,
    )
}

pub(super) fn push_popup_background(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let panel_radius = popup_panel_radius(metadata);
    let shadow = &popup_row_palette().shadow;
    // Keep shadows behind the exact popup background z used by hit projection.
    commands.push(quad_command(
        node_id,
        popup_shadow_frame(frame, 2.0, 2.0),
        clip_frame.clone(),
        z_index.saturating_sub(2),
        shadow,
        None,
        0.0,
        panel_radius + 2.0,
        PopupCommandPaintState::open_surface(),
        opacity * POPUP_SHADOW_FAR_OPACITY,
    ));
    commands.push(quad_command(
        node_id,
        popup_shadow_frame(frame, 1.0, 1.0),
        clip_frame.clone(),
        z_index.saturating_sub(1),
        shadow,
        None,
        0.0,
        panel_radius + 1.0,
        PopupCommandPaintState::open_surface(),
        opacity * POPUP_SHADOW_NEAR_OPACITY,
    ));
    commands.push(quad_command(
        node_id,
        frame,
        clip_frame,
        z_index,
        &popup_row_palette().background,
        Some(popup_row_palette().border.as_str()),
        popup_row_palette().border_width,
        panel_radius,
        PopupCommandPaintState::open_surface(),
        opacity,
    ));
}

fn popup_shadow_frame(frame: UiFrame, grow: f32, offset_y: f32) -> UiFrame {
    UiFrame::new(
        frame.x - grow,
        frame.y + offset_y - grow,
        frame.width + grow * 2.0,
        frame.height + grow * 2.0,
    )
}

fn popup_panel_radius(metadata: &UiTemplateNodeMetadata) -> f32 {
    ["corner_radius", "radius"]
        .into_iter()
        .find_map(|key| {
            metadata
                .style_overrides
                .get(key)
                .or_else(|| metadata.attributes.get(key))
                .and_then(|value| {
                    value
                        .as_float()
                        .or_else(|| value.as_integer().map(|value| value as f64))
                })
        })
        .map(|radius| radius as f32)
        .filter(|radius| radius.is_finite() && *radius >= 0.0)
        .unwrap_or(popup_row_palette().panel_radius)
}

pub(super) fn push_popup_row_surface(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    row_frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    state: PopupRowPaintState,
    opacity: f32,
) {
    if state.unavailable() {
        return;
    }
    let surface = if state.selected {
        Some((&popup_row_palette().selected_background, None, 0.0))
    } else if state.surface_hot() {
        Some((&popup_row_palette().hover_background, None, 0.0))
    } else if state.focused_only() {
        Some((
            &popup_row_palette().background,
            Some(popup_row_palette().border.as_str()),
            popup_row_palette().border_width,
        ))
    } else {
        None
    };
    if let Some((background, border, border_width)) = surface {
        commands.push(quad_command(
            node_id,
            row_frame,
            clip_frame,
            z_index,
            background,
            border,
            border_width,
            popup_row_palette().row_radius,
            state.command_state,
            opacity,
        ));
    }
    if state.selected {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                row_frame.x,
                row_frame.y + popup_row_palette().row_radius,
                popup_row_palette().selected_mark_width,
                (row_frame.height - popup_row_palette().row_radius * 2.0)
                    .max(popup_row_palette().min_frame_extent),
            ),
            clip_frame,
            z_index.saturating_add(1),
            &popup_row_palette().accent_text,
            None,
            0.0,
            popup_row_palette().border_width,
            state.command_state,
            opacity,
        ));
    }
}

pub(super) fn push_popup_separator(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    row_frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let state = PopupCommandPaintState::separator();
    commands.push(quad_command(
        node_id,
        UiFrame::new(
            row_frame.x + popup_row_palette().separator_inset,
            row_frame.y + row_frame.height * 0.5,
            (row_frame.width - popup_row_palette().separator_inset * 2.0)
                .max(popup_row_palette().min_frame_extent),
            popup_row_palette().border_width,
        ),
        clip_frame,
        z_index,
        &popup_row_palette().border,
        None,
        0.0,
        0.0,
        state,
        opacity,
    ));
}

pub(super) fn push_popup_row_label(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    row_frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    state: PopupRowPaintState,
    opacity: f32,
) {
    if text.is_empty() {
        return;
    }
    commands.push(text_command(
        node_id,
        UiFrame::new(
            row_frame.x + popup_row_palette().row_text_inset_x,
            row_frame.y + (row_frame.height - popup_row_palette().line_height).max(0.0) * 0.5,
            (row_frame.width - popup_row_palette().row_text_inset_x * 2.0)
                .max(popup_row_palette().min_frame_extent),
            row_frame.height.min(popup_row_palette().line_height),
        ),
        clip_frame,
        z_index,
        text,
        foreground,
        state.command_state,
        opacity,
    ));
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PopupRowPaintState {
    selected: bool,
    surface_hot: bool,
    command_state: PopupCommandPaintState,
}

impl PopupRowPaintState {
    pub(super) fn resolve(
        selected: bool,
        hovered: bool,
        focused: bool,
        pressed: bool,
        disabled: bool,
        loading: bool,
    ) -> Self {
        let family = UiPainterFamily::PopupRow;
        let surface_hot = hovered || pressed;
        let painter_state = UiPainterState {
            hovered,
            pressed,
            focused,
            focus_visible: focused,
            disabled,
            loading,
            checked: selected,
            selected,
            ..UiPainterState::normal()
        };
        Self {
            selected,
            surface_hot,
            command_state: PopupCommandPaintState {
                family,
                visual_state: painter_state.resolved_state_for_family(family),
            },
        }
    }

    pub(super) fn text_color(self, danger: bool) -> &'static str {
        if self.unavailable() {
            &popup_row_palette().muted_text
        } else if danger {
            &popup_row_palette().danger_text
        } else if self.selected || self.surface_hot() {
            &popup_row_palette().accent_text
        } else {
            &popup_row_palette().text
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.command_state.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn surface_hot(self) -> bool {
        self.surface_hot
            || matches!(
                self.command_state.visual_state,
                UiPainterResolvedState::Hovered
                    | UiPainterResolvedState::Pressed
                    | UiPainterResolvedState::Open
                    | UiPainterResolvedState::Dragging
                    | UiPainterResolvedState::DropHovered
            )
    }

    fn focused_only(self) -> bool {
        matches!(
            self.command_state.visual_state,
            UiPainterResolvedState::Focused
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct PopupCommandPaintState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
}

impl PopupCommandPaintState {
    fn open_surface() -> Self {
        Self {
            family: UiPainterFamily::Dropdown,
            visual_state: UiPainterResolvedState::Open,
        }
    }

    fn separator() -> Self {
        Self {
            family: UiPainterFamily::PopupRow,
            visual_state: UiPainterResolvedState::Normal,
        }
    }
}

fn option_row_height(
    metadata: &UiTemplateNodeMetadata,
    control_frame: UiFrame,
    row_count: usize,
) -> f32 {
    if metadata.component == "DropdownPopup" && row_count > 0 {
        return (control_frame.height / row_count as f32).max(popup_row_palette().row_min_height);
    }
    control_frame.height.max(popup_row_palette().row_min_height)
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: &str,
    border: Option<&str>,
    border_width: f32,
    corner_radius: f32,
    state: PopupCommandPaintState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.to_string()),
            border_color: border.map(str::to_string),
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

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    state: PopupCommandPaintState,
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
            font_size: popup_row_palette().font_size,
            line_height: popup_row_palette().line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_surface_and_rows_keep_distinct_radius_tiers() {
        let palette = popup_row_palette();
        let controls = EditorDesignTokens::workbench_dark().controls;
        let metadata = UiTemplateNodeMetadata {
            style_overrides: toml::from_str("corner_radius = 14.0").unwrap(),
            ..Default::default()
        };

        assert_eq!(palette.panel_radius, controls.panel_radius);
        assert_eq!(palette.row_radius, controls.small_radius);

        let mut commands = Vec::new();
        push_popup_background(
            &mut commands,
            UiNodeId(1),
            &metadata,
            UiFrame::new(0.0, 0.0, 160.0, 120.0),
            None,
            0,
            1.0,
        );

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].z_index, -2);
        assert_eq!(commands[1].z_index, -1);
        assert_eq!(commands[2].z_index, 0);
        assert_eq!(commands[2].style.corner_radius, 14.0);
        assert_eq!(commands[0].frame, UiFrame::new(-2.0, 0.0, 164.0, 124.0));
        assert_eq!(commands[1].frame, UiFrame::new(-1.0, 0.0, 162.0, 122.0));
    }

    #[test]
    fn popup_shadow_layers_stay_behind_the_authoritative_background() {
        let frame = UiFrame::new(10.0, 20.0, 160.0, 120.0);

        assert_eq!(
            popup_shadow_frame(frame, 2.0, 2.0),
            UiFrame::new(8.0, 20.0, 164.0, 124.0)
        );
        assert_eq!(
            popup_shadow_frame(frame, 1.0, 1.0),
            UiFrame::new(9.0, 20.0, 162.0, 122.0)
        );
    }

    #[test]
    fn popup_attribute_id_set_borrows_small_inputs_and_indexes_large_inputs() {
        let small = toml::Value::Array(vec![
            toml::Value::String("first".to_string()),
            toml::Value::String("second".to_string()),
        ]);
        let small_set = PopupAttributeIdSet::new(Some(&small));
        assert!(matches!(small_set, PopupAttributeIdSet::Linear(_)));
        assert!(small_set.contains("second"));
        assert!(!small_set.contains("missing"));

        let large = toml::Value::Array(
            (0..=POPUP_ATTRIBUTE_LINEAR_SCAN_LIMIT)
                .map(|index| toml::Value::String(format!("item-{index}")))
                .collect(),
        );
        let large_set = PopupAttributeIdSet::new(Some(&large));
        assert!(matches!(large_set, PopupAttributeIdSet::Indexed(_)));
        assert!(large_set.contains("item-8"));
        assert!(!large_set.contains("missing"));
    }

    #[test]
    fn popup_row_frame_consumes_authored_padding_and_spacing() {
        let metadata = UiTemplateNodeMetadata {
            attributes: toml::from_str(
                r#"
layout_padding_left = 8.0
layout_padding_right = 8.0
layout_padding_top = 4.0
layout_padding_bottom = 4.0
layout_spacing = 4.0
"#,
            )
            .unwrap(),
            ..Default::default()
        };
        let popup = UiFrame::new(10.0, 20.0, 190.0, 148.0);

        assert_frame_near(
            popup_row_frame(&metadata, popup, 5, 0).expect("first row"),
            UiFrame::new(18.0, 24.0, 174.0, 24.8),
        );
        assert_frame_near(
            popup_row_frame(&metadata, popup, 5, 4).expect("last row"),
            UiFrame::new(18.0, 139.2, 174.0, 24.8),
        );
        assert!((menu_row_height(&metadata, popup, 5).unwrap() - 24.8).abs() <= 0.001);
        assert!((popup_rows_height(&metadata, 5, 24.8).unwrap() - 148.0).abs() <= 0.001);
    }

    fn assert_frame_near(actual: UiFrame, expected: UiFrame) {
        for (actual, expected) in [
            (actual.x, expected.x),
            (actual.y, expected.y),
            (actual.width, expected.width),
            (actual.height, expected.height),
        ] {
            assert!((actual - expected).abs() <= 0.001, "{actual} != {expected}");
        }
    }
}
