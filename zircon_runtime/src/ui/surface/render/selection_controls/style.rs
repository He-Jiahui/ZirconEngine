use std::sync::OnceLock;

use zircon_runtime_interface::ui::{
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    style::UiRgbaColor,
    tree::UiTemplateNodeMetadata,
};

use super::{
    metadata::{first_rgba_attribute, line_height, metric_attribute},
    state::SelectionRenderState,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct SelectionVisual {
    pub(super) label: UiRgbaColor,
    pub(super) label_disabled: UiRgbaColor,
    pub(super) mark_idle_fill: UiRgbaColor,
    pub(super) mark_idle_border: UiRgbaColor,
    pub(super) mark_disabled_fill: UiRgbaColor,
    pub(super) mark_disabled_border: UiRgbaColor,
    pub(super) selected_surface: UiRgbaColor,
    pub(super) accent: UiRgbaColor,
    pub(super) radio_checked_fill: UiRgbaColor,
    pub(super) radio_checked_border: UiRgbaColor,
    pub(super) toggle_idle: UiRgbaColor,
    pub(super) toggle_thumb_idle: UiRgbaColor,
    pub(super) toggle_thumb_active: UiRgbaColor,
    pub(super) toggle_hover: UiRgbaColor,
    pub(super) toggle_pressed: UiRgbaColor,
    pub(super) mark_inset_x: f32,
    pub(super) mark_size: f32,
    pub(super) label_gap: f32,
    pub(super) label_inset_y: f32,
    pub(super) label_font_size: f32,
    pub(super) label_line_height: f32,
    pub(super) radio_dot_size: f32,
    pub(super) toggle_track_width: f32,
    pub(super) toggle_track_height: f32,
    pub(super) toggle_thumb_size: f32,
    pub(super) toggle_right_inset: f32,
    pub(super) toggle_thumb_inset: f32,
    pub(super) border_width: f32,
    pub(super) mark_radius: f32,
    pub(super) min_frame_extent: f32,
}

impl SelectionVisual {
    pub(super) fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_selection_visual();
        visual.label = first_rgba_attribute(metadata, &["label_color", "foreground_color"])
            .unwrap_or(visual.label);
        visual.label_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.label_disabled);
        visual.mark_idle_fill =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.mark_idle_fill);
        visual.mark_idle_border =
            first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.mark_idle_border);
        visual.mark_disabled_fill = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.mark_disabled_fill);
        visual.mark_disabled_border = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.mark_disabled_border);
        visual.selected_surface = first_rgba_attribute(metadata, &["selected_background_color"])
            .unwrap_or(visual.selected_surface);
        visual.accent = first_rgba_attribute(metadata, &["accent_color", "focus_border_color"])
            .unwrap_or(visual.accent);
        visual.radio_checked_fill = first_rgba_attribute(metadata, &["checked_background_color"])
            .unwrap_or(visual.radio_checked_fill);
        visual.radio_checked_border = first_rgba_attribute(metadata, &["checked_border_color"])
            .unwrap_or(visual.radio_checked_border);
        visual.toggle_idle =
            first_rgba_attribute(metadata, &["toggle_background_color", "background_color"])
                .unwrap_or(visual.toggle_idle);
        visual.toggle_thumb_idle =
            first_rgba_attribute(metadata, &["thumb_color", "foreground_color"])
                .unwrap_or(visual.toggle_thumb_idle);
        visual.toggle_thumb_active = first_rgba_attribute(metadata, &["selected_thumb_color"])
            .unwrap_or(visual.toggle_thumb_active);
        visual.toggle_hover = first_rgba_attribute(metadata, &["hover_background_color"])
            .unwrap_or(visual.toggle_hover);
        visual.toggle_pressed = first_rgba_attribute(metadata, &["pressed_background_color"])
            .unwrap_or(visual.toggle_pressed);
        visual.mark_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.mark_size);
        visual.label_gap = metric_attribute(metadata, "layout_spacing")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.label_gap);
        visual.radio_dot_size = metric_attribute(metadata, "dot_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.radio_dot_size);
        visual.toggle_track_width = metric_attribute(metadata, "track_width")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.toggle_track_width);
        visual.toggle_track_height = metric_attribute(metadata, "track_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.toggle_track_height);
        visual.toggle_thumb_size = metric_attribute(metadata, "thumb_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.toggle_thumb_size);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.mark_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.mark_radius);
        visual.label_font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.label_font_size);
        visual.label_line_height = line_height(
            metadata,
            "line_height",
            "line_height_ratio",
            visual.label_font_size,
            visual.label_line_height,
        );
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_selection_visual() -> &'static SelectionVisual {
    static VISUAL: OnceLock<SelectionVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        let border = controls.border_width;
        let track_height = controls.dense_height - density.gap_medium - border * 2.0;
        SelectionVisual {
            label: colors.text_secondary,
            label_disabled: colors.text_disabled,
            mark_idle_fill: colors.surface_recessed,
            mark_idle_border: colors.separator_strong,
            mark_disabled_fill: colors.surface_disabled,
            mark_disabled_border: colors.border_disabled,
            selected_surface: colors.surface_selected,
            accent: colors.accent,
            radio_checked_fill: colors.surface[2],
            radio_checked_border: colors.border,
            toggle_idle: colors.surface[2],
            toggle_thumb_idle: colors.text_secondary,
            toggle_thumb_active: colors.text_primary,
            toggle_hover: colors.surface_hover,
            toggle_pressed: colors.surface[3],
            mark_inset_x: density.gap_medium + border * 2.0,
            mark_size: controls.dense_height - density.gap_large,
            label_gap: density.gap_medium + border,
            label_inset_y: density.gap_small + border,
            label_font_size: typography.body_size,
            label_line_height: typography.body_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            radio_dot_size: density.gap_small + border * 3.0,
            toggle_track_width: controls.default_height + border * 2.0,
            toggle_track_height: track_height,
            toggle_thumb_size: track_height - density.gap_small - border * 2.0,
            toggle_right_inset: density.gap_medium,
            toggle_thumb_inset: border * 2.0,
            border_width: border,
            mark_radius: controls.small_radius,
            min_frame_extent: border.max(f32::EPSILON),
        }
    })
}

pub(super) fn checkbox_background(
    state: &SelectionRenderState,
    visual: &SelectionVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_fill
    } else if state.active() {
        visual.selected_surface
    } else {
        visual.mark_idle_fill
    }
}

pub(super) fn checkbox_border(
    state: &SelectionRenderState,
    visual: &SelectionVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_border
    } else if state.focus_border() || state.active() {
        visual.accent
    } else {
        visual.mark_idle_border
    }
}

pub(super) fn radio_background(
    state: &SelectionRenderState,
    visual: &SelectionVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_fill
    } else if state.active() {
        visual.radio_checked_fill
    } else {
        visual.mark_idle_fill
    }
}

pub(super) fn radio_border(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_border
    } else if state.focus_border() {
        visual.accent
    } else if state.active() {
        visual.radio_checked_border
    } else {
        visual.mark_idle_border
    }
}

pub(super) fn radio_dot(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.label_disabled
    } else {
        visual.accent
    }
}

pub(super) fn toggle_track(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_fill
    } else if state.active() {
        visual.selected_surface
    } else if state.pressed() {
        visual.toggle_pressed
    } else if state.surface_hot() {
        visual.toggle_hover
    } else {
        visual.toggle_idle
    }
}

pub(super) fn toggle_border(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.mark_disabled_border
    } else if state.focus_border() || state.active() {
        visual.accent
    } else {
        visual.mark_idle_border
    }
}

pub(super) fn toggle_thumb(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.label_disabled
    } else if state.active() {
        visual.toggle_thumb_active
    } else {
        visual.toggle_thumb_idle
    }
}

pub(super) fn label_color(state: &SelectionRenderState, visual: &SelectionVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.label_disabled
    } else {
        visual.label
    }
}
