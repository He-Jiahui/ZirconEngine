use std::sync::OnceLock;

use zircon_runtime_interface::ui::{
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::UiNodeId,
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiPainterState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::popup_position::{
    anchored_popup_frame, has_popup_position_metadata, popup_anchor_frame, popup_layout_bounds,
    PopupPlacement,
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
            row_radius: controls.small_radius,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
        }
    })
}

const POPUP_RENDER_Z_OFFSET: i32 = 100;

pub(super) fn popup_base_z(z_index: i32) -> i32 {
    z_index.saturating_add(POPUP_RENDER_Z_OFFSET)
}

pub(super) fn menu_row_height(frame: UiFrame, row_count: usize) -> Option<f32> {
    (row_count > 0)
        .then_some((frame.height / row_count as f32).max(popup_row_palette().row_min_height))
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
    let anchor_frame = popup_anchor_frame(metadata, control_frame);
    anchored_popup_frame(
        metadata,
        anchor_frame,
        control_frame
            .width
            .max(popup_row_palette().min_frame_extent),
        row_height * row_count as f32,
        bounds,
        PopupPlacement::BottomStart,
        popup_row_palette().option_row_gap,
    )
}

pub(super) fn push_popup_background(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let state = PopupCommandPaintState::open_surface();
    commands.push(quad_command(
        node_id,
        frame,
        clip_frame,
        z_index,
        &popup_row_palette().background,
        Some(popup_row_palette().border.as_str()),
        popup_row_palette().border_width,
        popup_row_palette().row_radius,
        state,
        opacity,
    ));
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
