use std::sync::OnceLock;

use zircon_runtime_interface::ui::{
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    style::{UiPainterFamily, UiRgbaColor},
    tree::UiTemplateNodeMetadata,
};

use super::{
    metadata::{ButtonKind, first_rgba_attribute, line_height, metric_attribute},
    state::ButtonRenderState,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct ButtonVisual {
    pub(super) primary_surface: UiRgbaColor,
    pub(super) primary_hover: UiRgbaColor,
    pub(super) primary_pressed: UiRgbaColor,
    pub(super) primary_border: UiRgbaColor,
    pub(super) primary_text: UiRgbaColor,
    pub(super) secondary_surface: UiRgbaColor,
    pub(super) secondary_hover: UiRgbaColor,
    pub(super) secondary_pressed: UiRgbaColor,
    pub(super) secondary_border: UiRgbaColor,
    pub(super) secondary_text: UiRgbaColor,
    pub(super) tertiary_surface: UiRgbaColor,
    pub(super) tertiary_hover: UiRgbaColor,
    pub(super) tertiary_pressed: UiRgbaColor,
    pub(super) tertiary_text: UiRgbaColor,
    pub(super) danger_surface: UiRgbaColor,
    pub(super) danger_border: UiRgbaColor,
    pub(super) danger_text: UiRgbaColor,
    pub(super) disabled_surface: UiRgbaColor,
    pub(super) disabled_border: UiRgbaColor,
    pub(super) disabled_text: UiRgbaColor,
    pub(super) focus_border: UiRgbaColor,
    pub(super) icon_normal: UiRgbaColor,
    pub(super) icon_selected_surface: UiRgbaColor,
    pub(super) icon_selected: UiRgbaColor,
    pub(super) icon_panel_surface: UiRgbaColor,
    pub(super) icon_panel_border: UiRgbaColor,
    pub(super) selected_background: Option<UiRgbaColor>,
    pub(super) padding_left: f32,
    pub(super) padding_right: f32,
    pub(super) icon_size: f32,
    pub(super) icon_button_size: f32,
    pub(super) spacing: f32,
    pub(super) border_width: f32,
    pub(super) button_radius: f32,
    pub(super) icon_button_radius: f32,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) min_frame_extent: f32,
}

impl ButtonVisual {
    pub(super) fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_button_visual();
        if let Some(color) = first_rgba_attribute(metadata, &["background_color"]) {
            visual.primary_surface = color;
            visual.secondary_surface = color;
            visual.tertiary_surface = color;
            visual.danger_surface = color;
            visual.icon_panel_surface = color;
        }
        if let Some(color) = first_rgba_attribute(metadata, &["hover_background_color"]) {
            visual.primary_hover = color;
            visual.secondary_hover = color;
            visual.tertiary_hover = color;
        }
        if let Some(color) = first_rgba_attribute(metadata, &["pressed_background_color"]) {
            visual.primary_pressed = color;
            visual.secondary_pressed = color;
            visual.tertiary_pressed = color;
        }
        visual.selected_background = first_rgba_attribute(metadata, &["selected_background_color"]);
        visual.disabled_surface = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.disabled_surface);

        if let Some(color) = first_rgba_attribute(metadata, &["border_color"]) {
            visual.primary_border = color;
            visual.secondary_border = color;
            visual.danger_border = color;
            visual.icon_panel_border = color;
        }
        visual.focus_border =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.focus_border);
        visual.disabled_border = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.disabled_border);

        if let Some(color) = first_rgba_attribute(metadata, &["foreground_color", "text_color"]) {
            visual.primary_text = color;
            visual.secondary_text = color;
            visual.tertiary_text = color;
            visual.danger_text = color;
            visual.icon_normal = color;
        }
        visual.icon_normal =
            first_rgba_attribute(metadata, &["icon_color"]).unwrap_or(visual.icon_normal);
        visual.icon_selected =
            first_rgba_attribute(metadata, &["selected_icon_color", "icon_color"])
                .unwrap_or(visual.icon_selected);
        visual.disabled_text = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.disabled_text);

        visual.padding_left = metric_attribute(metadata, "layout_padding_left")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_left);
        visual.padding_right = metric_attribute(metadata, "layout_padding_right")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_right);
        visual.icon_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.icon_size);
        visual.icon_button_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.icon_button_size);
        visual.spacing = metric_attribute(metadata, "layout_spacing")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.spacing);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        if let Some(radius) = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
        {
            visual.button_radius = radius;
            visual.icon_button_radius = radius;
        }
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
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_button_visual() -> &'static ButtonVisual {
    static VISUAL: OnceLock<ButtonVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        ButtonVisual {
            primary_surface: colors.accent_soft,
            primary_hover: colors.surface_selected,
            primary_pressed: colors.surface[3],
            primary_border: colors.accent,
            primary_text: colors.text_primary,
            secondary_surface: colors.surface[1],
            secondary_hover: colors.surface_hover,
            secondary_pressed: colors.surface[3],
            secondary_border: colors.border,
            secondary_text: colors.text_primary,
            tertiary_surface: colors.surface[0],
            tertiary_hover: colors.surface_hover,
            tertiary_pressed: colors.surface[3],
            tertiary_text: colors.text_secondary,
            danger_surface: colors.error_container,
            danger_border: colors.error,
            danger_text: colors.error,
            disabled_surface: colors.surface_disabled,
            disabled_border: colors.border_disabled,
            disabled_text: colors.text_disabled,
            focus_border: colors.accent,
            icon_normal: colors.text_secondary,
            icon_selected_surface: colors.surface_selected,
            icon_selected: colors.accent,
            icon_panel_surface: colors.surface[2],
            icon_panel_border: colors.border,
            selected_background: None,
            padding_left: density.gap_large,
            padding_right: density.gap_large,
            icon_size: controls.dense_height - density.gap_large,
            icon_button_size: controls.dense_height - density.gap_large
                + controls.border_width * 2.0,
            spacing: density.gap_medium,
            border_width: controls.border_width,
            button_radius: controls.small_radius,
            icon_button_radius: controls.control_radius,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn background_color(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else if state.family() == UiPainterFamily::IconButton {
        if state.selected() {
            visual
                .selected_background
                .unwrap_or(visual.icon_selected_surface)
        } else if state.pressed() {
            visual.secondary_pressed
        } else if state.surface_hot() {
            visual.secondary_hover
        } else {
            visual.icon_panel_surface
        }
    } else if state.pressed() {
        surface_for_kind(state.kind(), visual, ButtonSurfaceState::Pressed)
    } else if state.selected() {
        visual
            .selected_background
            .unwrap_or_else(|| surface_for_kind(state.kind(), visual, ButtonSurfaceState::Hover))
    } else if state.surface_hot() {
        surface_for_kind(state.kind(), visual, ButtonSurfaceState::Hover)
    } else {
        surface_for_kind(state.kind(), visual, ButtonSurfaceState::Normal)
    }
}

#[derive(Clone, Copy)]
enum ButtonSurfaceState {
    Normal,
    Hover,
    Pressed,
}

fn surface_for_kind(
    kind: ButtonKind,
    visual: &ButtonVisual,
    state: ButtonSurfaceState,
) -> UiRgbaColor {
    match (kind, state) {
        (ButtonKind::Primary, ButtonSurfaceState::Normal) => visual.primary_surface,
        (ButtonKind::Primary, ButtonSurfaceState::Hover) => visual.primary_hover,
        (ButtonKind::Primary, ButtonSurfaceState::Pressed) => visual.primary_pressed,
        (ButtonKind::Secondary, ButtonSurfaceState::Normal) => visual.secondary_surface,
        (ButtonKind::Secondary, ButtonSurfaceState::Hover) => visual.secondary_hover,
        (ButtonKind::Secondary, ButtonSurfaceState::Pressed) => visual.secondary_pressed,
        (ButtonKind::Tertiary, ButtonSurfaceState::Normal) => visual.tertiary_surface,
        (ButtonKind::Tertiary, ButtonSurfaceState::Hover) => visual.tertiary_hover,
        (ButtonKind::Tertiary, ButtonSurfaceState::Pressed) => visual.tertiary_pressed,
        (ButtonKind::Danger, _) => visual.danger_surface,
    }
}

pub(super) fn border_color(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_border
    } else if state.focused() || state.pressed() || state.selected() {
        visual.focus_border
    } else if state.family() == UiPainterFamily::IconButton {
        visual.icon_panel_border
    } else {
        match state.kind() {
            ButtonKind::Primary => visual.primary_border,
            ButtonKind::Danger => visual.danger_border,
            ButtonKind::Secondary | ButtonKind::Tertiary => visual.secondary_border,
        }
    }
}

pub(super) fn foreground_color(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else {
        match state.kind() {
            ButtonKind::Primary => visual.primary_text,
            ButtonKind::Danger => visual.danger_text,
            ButtonKind::Tertiary => visual.tertiary_text,
            ButtonKind::Secondary => visual.secondary_text,
        }
    }
}

pub(super) fn icon_button_foreground(
    state: &ButtonRenderState,
    visual: &ButtonVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else if state.selected() || state.pressed() {
        visual.icon_selected
    } else {
        visual.icon_normal
    }
}
