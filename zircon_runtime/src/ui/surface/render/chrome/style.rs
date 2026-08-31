use std::{borrow::Cow, sync::OnceLock};

use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens,
    style::{UiPainterResolvedState, UiRgbaColor},
    tree::UiTemplateNodeMetadata,
};

use super::{
    metadata::{ChromeKind, color_attribute, number_attribute},
    state::ChromeRenderState,
};

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

pub(super) fn surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    kind: ChromeKind,
    state: &ChromeRenderState,
) -> Cow<'a, str> {
    if state.visual_state() == UiPainterResolvedState::Disabled {
        Cow::Borrowed(&chrome_palette().surface_disabled)
    } else if state.visual_state() == UiPainterResolvedState::Loading {
        Cow::Borrowed(&chrome_palette().surface_loading)
    } else if state.visual_state() == UiPainterResolvedState::Pressed {
        color_attribute(metadata, "pressed_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().surface_pressed))
    } else if state.visual_state() == UiPainterResolvedState::Open {
        color_attribute(metadata, "open_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&chrome_palette().surface_open))
    } else if state.visual_state() == UiPainterResolvedState::Hovered {
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

pub(super) fn border_color<'a>(
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

pub(super) fn separator_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ChromeRenderState,
) -> Cow<'a, str> {
    color_attribute(metadata, "separator_color")
        .map(Cow::Borrowed)
        .unwrap_or_else(|| {
            if state.active() {
                Cow::Borrowed(&chrome_palette().border_active)
            } else {
                Cow::Borrowed(&chrome_palette().border_muted)
            }
        })
}

pub(super) fn text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ChromeRenderState,
) -> Cow<'a, str> {
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

pub(super) fn icon_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &ChromeRenderState,
) -> Cow<'a, str> {
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

pub(super) fn border_width(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> f32 {
    number_attribute(metadata, "border_width").unwrap_or_else(|| match kind {
        ChromeKind::Viewport => 0.0,
        _ => chrome_palette().border_width,
    })
}

pub(super) fn corner_radius(metadata: &UiTemplateNodeMetadata, kind: ChromeKind) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or_else(|| match kind {
            ChromeKind::Shell | ChromeKind::Toolbar | ChromeKind::StatusBar => 0.0,
            _ => chrome_palette().radius_small,
        })
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
