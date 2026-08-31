use std::sync::OnceLock;

use zircon_runtime_interface::ui::{
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    style::UiRgbaColor,
    tree::UiTemplateNodeMetadata,
};

use super::{
    metadata::{first_rgba_attribute, line_height, metric_attribute},
    state::SegmentedRenderState,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct SegmentedVisual {
    pub(super) background: UiRgbaColor,
    pub(super) border: UiRgbaColor,
    pub(super) selected_surface: UiRgbaColor,
    pub(super) focus_border: UiRgbaColor,
    pub(super) selected_border: UiRgbaColor,
    pub(super) selected_underline: UiRgbaColor,
    pub(super) hover: UiRgbaColor,
    pub(super) pressed: UiRgbaColor,
    pub(super) disabled_surface: UiRgbaColor,
    pub(super) disabled_border: UiRgbaColor,
    pub(super) text: UiRgbaColor,
    pub(super) text_muted: UiRgbaColor,
    pub(super) text_disabled: UiRgbaColor,
    pub(super) group_label: UiRgbaColor,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) group_label_font_size: f32,
    pub(super) group_label_line_height: f32,
    pub(super) group_label_height: f32,
    pub(super) group_label_gap: f32,
    pub(super) segment_text_inset_x: f32,
    pub(super) segment_text_inset_y: f32,
    pub(super) selected_inset: f32,
    pub(super) corner_radius: f32,
    pub(super) tab_font_size: f32,
    pub(super) tab_line_height: f32,
    pub(super) tab_text_inset_x: f32,
    pub(super) tab_underline_height: f32,
    pub(super) border_width: f32,
    pub(super) selected_border_width: f32,
    pub(super) min_frame_extent: f32,
}

impl SegmentedVisual {
    pub(super) fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_segmented_visual();
        visual.background =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.background);
        visual.border = first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.border);
        visual.selected_surface = first_rgba_attribute(metadata, &["selected_background_color"])
            .unwrap_or(visual.selected_surface);
        visual.focus_border =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.focus_border);
        visual.selected_border = first_rgba_attribute(metadata, &["selected_border_color"])
            .unwrap_or(visual.selected_border);
        visual.selected_underline =
            first_rgba_attribute(metadata, &["selected_underline_color", "accent_color"])
                .unwrap_or(visual.selected_underline);
        visual.hover =
            first_rgba_attribute(metadata, &["hover_background_color"]).unwrap_or(visual.hover);
        visual.pressed =
            first_rgba_attribute(metadata, &["pressed_background_color"]).unwrap_or(visual.pressed);
        visual.disabled_surface = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.disabled_surface);
        visual.disabled_border = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.disabled_border);
        visual.text = first_rgba_attribute(
            metadata,
            &["selected_foreground_color", "selected_text_color"],
        )
        .unwrap_or(visual.text);
        visual.text_muted =
            first_rgba_attribute(metadata, &["foreground_color", "idle_text_color"])
                .unwrap_or(visual.text_muted);
        visual.text_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.text_disabled);
        visual.group_label =
            first_rgba_attribute(metadata, &["label_color"]).unwrap_or(visual.group_label);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.selected_border_width = metric_attribute(metadata, "selected_border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.selected_border_width);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        visual.font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.font_size);
        visual.line_height = line_height(
            metadata,
            "line_height",
            "line_height_ratio",
            visual.font_size,
            visual.line_height,
        );
        visual.tab_font_size = metric_attribute(metadata, "tab_font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.tab_font_size);
        visual.tab_line_height = line_height(
            metadata,
            "tab_line_height",
            "tab_line_height_ratio",
            visual.tab_font_size,
            visual.tab_line_height,
        );
        visual.group_label_height = metric_attribute(metadata, "group_label_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.group_label_height);
        visual.group_label_gap = metric_attribute(metadata, "group_label_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.group_label_gap);
        visual.segment_text_inset_x = metric_attribute(metadata, "segment_text_inset_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.segment_text_inset_x);
        visual.segment_text_inset_y = metric_attribute(metadata, "segment_text_inset_y")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.segment_text_inset_y);
        visual.selected_inset = metric_attribute(metadata, "selected_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.selected_inset);
        visual.tab_text_inset_x = metric_attribute(metadata, "tab_text_inset_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.tab_text_inset_x);
        visual.tab_underline_height = metric_attribute(metadata, "selected_underline_height")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.tab_underline_height);
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_segmented_visual() -> &'static SegmentedVisual {
    static VISUAL: OnceLock<SegmentedVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        SegmentedVisual {
            background: colors.surface[2],
            border: colors.border,
            selected_surface: colors.surface_selected,
            focus_border: colors.accent,
            selected_border: colors.accent,
            selected_underline: colors.accent,
            hover: colors.surface_hover,
            pressed: colors.surface[3],
            disabled_surface: colors.surface_disabled,
            disabled_border: colors.border_disabled,
            text: colors.text_primary,
            text_muted: colors.text_secondary,
            text_disabled: colors.text_disabled,
            group_label: colors.text_secondary,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            group_label_font_size: typography.caption_size,
            group_label_line_height: typography.caption_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            group_label_height: typography.overlay_size + controls.border_width * 2.0,
            group_label_gap: density.gap_small,
            segment_text_inset_x: density.gap_medium,
            segment_text_inset_y: density.gap_small + controls.border_width,
            selected_inset: controls.border_width * 2.0,
            corner_radius: controls.control_radius,
            tab_font_size: typography.overlay_size,
            tab_line_height: typography.overlay_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            tab_text_inset_x: density.gap_large,
            tab_underline_height: controls.border_width * 2.0,
            border_width: controls.border_width,
            selected_border_width: 0.0,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn segmented_background(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else if state.pressed() {
        visual.pressed
    } else if state.surface_hot() {
        visual.hover
    } else {
        visual.background
    }
}

pub(super) fn segmented_border(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_border
    } else if state.pressed() || state.focused() || state.surface_hot() {
        visual.focus_border
    } else {
        visual.border
    }
}

pub(super) fn divider_color(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    segmented_border(state, visual)
}

pub(super) fn selected_surface(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else {
        visual.selected_surface
    }
}

pub(super) fn selected_underline(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.selected_underline
    }
}

pub(super) fn option_text_color(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    selected: bool,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if selected {
        visual.text
    } else {
        visual.text_muted
    }
}

pub(super) fn group_label_color(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.group_label
    }
}

pub(super) fn tab_background(
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> Option<UiRgbaColor> {
    if state.unavailable() {
        Some(visual.disabled_surface)
    } else if state.pressed() {
        Some(visual.pressed)
    } else if state.surface_hot() {
        Some(visual.hover)
    } else {
        first_rgba_attribute(metadata, &["background_color"])
    }
}

pub(super) fn tab_text_color(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.active() {
        visual.text
    } else {
        visual.text_muted
    }
}
