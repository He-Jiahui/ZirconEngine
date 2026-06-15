use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiPainterState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::popup_position::{
    anchored_popup_frame, has_popup_position_metadata, popup_anchor_frame, popup_layout_bounds,
    PopupPlacement,
};

const POPUP_BACKGROUND: &str = "#151b1f";
const POPUP_BORDER: &str = "#303840";
const POPUP_SELECTED_BACKGROUND: &str = "#12383d";
const POPUP_HOVER_BACKGROUND: &str = "#1a2429";
const POPUP_TEXT: &str = "#c5d0d5";
const POPUP_MUTED_TEXT: &str = "#59656c";
const POPUP_ACCENT_TEXT: &str = "#35c7d0";
const POPUP_DANGER_TEXT: &str = "#f25f52";
const POPUP_ROW_MIN_HEIGHT: f32 = 24.0;
const POPUP_ROW_TEXT_X: f32 = 9.0;
const POPUP_ROW_TEXT_Y: f32 = 5.0;
const POPUP_SEPARATOR_INSET: f32 = 8.0;
const POPUP_SELECTED_MARK_WIDTH: f32 = 3.0;
const POPUP_OPTION_ROW_GAP: f32 = 4.0;
const POPUP_RENDER_Z_OFFSET: i32 = 100;

pub(super) fn popup_base_z(z_index: i32) -> i32 {
    z_index.saturating_add(POPUP_RENDER_Z_OFFSET)
}

pub(super) fn menu_row_height(frame: UiFrame, row_count: usize) -> Option<f32> {
    (row_count > 0).then_some((frame.height / row_count as f32).max(POPUP_ROW_MIN_HEIGHT))
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
        control_frame.width.max(1.0),
        row_height * row_count as f32,
        bounds,
        PopupPlacement::BottomStart,
        POPUP_OPTION_ROW_GAP,
    )
}

pub(super) fn option_row_frame_within(
    metadata: &UiTemplateNodeMetadata,
    control_frame: UiFrame,
    row_count: usize,
    row: usize,
    bounds: Option<UiFrame>,
) -> Option<UiFrame> {
    if row >= row_count {
        return None;
    }
    let popup = option_popup_frame_within(metadata, control_frame, row_count, bounds)?;
    let row_height = option_row_height(metadata, control_frame, row_count);
    Some(UiFrame::new(
        popup.x,
        popup.y + row as f32 * row_height,
        popup.width,
        row_height,
    ))
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
        POPUP_BACKGROUND,
        Some(POPUP_BORDER),
        1.0,
        5.0,
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
    let background = if state.selected {
        Some(POPUP_SELECTED_BACKGROUND)
    } else if state.hot() {
        Some(POPUP_HOVER_BACKGROUND)
    } else {
        None
    };
    if let Some(background) = background {
        commands.push(quad_command(
            node_id,
            row_frame,
            clip_frame,
            z_index,
            background,
            None,
            0.0,
            3.0,
            state.command_state,
            opacity,
        ));
    }
    if state.selected {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                row_frame.x,
                row_frame.y + 4.0,
                POPUP_SELECTED_MARK_WIDTH,
                (row_frame.height - 8.0).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(1),
            POPUP_ACCENT_TEXT,
            None,
            0.0,
            1.0,
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
            row_frame.x + POPUP_SEPARATOR_INSET,
            row_frame.y + row_frame.height * 0.5,
            (row_frame.width - POPUP_SEPARATOR_INSET * 2.0).max(1.0),
            1.0,
        ),
        clip_frame,
        z_index,
        POPUP_BORDER,
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
            row_frame.x + POPUP_ROW_TEXT_X,
            row_frame.y + POPUP_ROW_TEXT_Y,
            (row_frame.width - POPUP_ROW_TEXT_X * 2.0).max(1.0),
            (row_frame.height - POPUP_ROW_TEXT_Y * 2.0).max(12.0),
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
            command_state: PopupCommandPaintState {
                family,
                visual_state: painter_state.resolved_state_for_family(family),
            },
        }
    }

    pub(super) fn text_color(self, danger: bool) -> &'static str {
        if self.unavailable() {
            POPUP_MUTED_TEXT
        } else if danger {
            POPUP_DANGER_TEXT
        } else if self.selected || self.hot() {
            POPUP_ACCENT_TEXT
        } else {
            POPUP_TEXT
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.command_state.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn hot(self) -> bool {
        matches!(
            self.command_state.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Pressed
                | UiPainterResolvedState::Focused
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
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
        return (control_frame.height / row_count as f32).max(POPUP_ROW_MIN_HEIGHT);
    }
    control_frame.height.max(POPUP_ROW_MIN_HEIGHT)
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
            font_size: 12.0,
            line_height: 14.4,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
