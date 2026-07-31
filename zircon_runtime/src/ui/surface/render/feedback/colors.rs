use std::{borrow::Cow, sync::OnceLock};

use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens, style::UiRgbaColor, tree::UiTemplateNodeMetadata,
};

use super::{color_attribute, state::FeedbackRenderState};

#[derive(Clone, Debug)]
struct FeedbackPalette {
    tooltip_surface: String,
    tooltip_border: String,
    tooltip_title: String,
    tooltip_body: String,
    tooltip_icon: String,
    alert_info_surface: String,
    alert_info_border: String,
    alert_info_mark: String,
    alert_success_surface: String,
    alert_success_border: String,
    alert_success_mark: String,
    alert_warning_surface: String,
    alert_warning_border: String,
    alert_warning_mark: String,
    alert_error_surface: String,
    alert_error_border: String,
    alert_error_mark: String,
    toast_surface: String,
    toast_surface_hover: String,
    toast_surface_pressed: String,
    toast_border: String,
    toast_text: String,
    toast_action: String,
    disabled_surface: String,
    disabled_border: String,
    disabled_text: String,
    focus_border: String,
}

fn feedback_palette() -> &'static FeedbackPalette {
    static PALETTE: OnceLock<FeedbackPalette> = OnceLock::new();
    PALETTE.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let palette = &tokens.palette;
        FeedbackPalette {
            tooltip_surface: css_color(palette.popup),
            tooltip_border: css_color(palette.separator_soft),
            tooltip_title: css_color(palette.text_primary),
            tooltip_body: css_color(palette.text_secondary),
            tooltip_icon: css_color(palette.accent),
            alert_info_surface: css_color(palette.info_container),
            alert_info_border: css_color(palette.info),
            alert_info_mark: css_color(palette.info),
            alert_success_surface: css_color(palette.success_container),
            alert_success_border: css_color(palette.success),
            alert_success_mark: css_color(palette.success),
            alert_warning_surface: css_color(palette.warning_container),
            alert_warning_border: css_color(palette.warning),
            alert_warning_mark: css_color(palette.warning),
            alert_error_surface: css_color(palette.error_container),
            alert_error_border: css_color(palette.error),
            alert_error_mark: css_color(palette.error),
            toast_surface: css_color(palette.accent_soft),
            toast_surface_hover: css_color(palette.surface_hover),
            toast_surface_pressed: css_color(palette.surface[3]),
            toast_border: css_color(palette.separator_soft),
            toast_text: css_color(palette.text_primary),
            toast_action: css_color(palette.accent),
            disabled_surface: css_color(palette.surface_disabled),
            disabled_border: css_color(palette.border_disabled),
            disabled_text: css_color(palette.text_disabled),
            focus_border: css_color(palette.accent),
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AlertTone {
    Info,
    Success,
    Warning,
    Error,
}

pub(super) fn alert_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_surface)
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_surface(tone))
    } else if state.pointer_hot() {
        color_attribute(metadata, "hover_background_color")
            .or_else(|| color_attribute(metadata, "background_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_surface(tone))
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_surface(tone))
    }
}

pub(super) fn alert_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_border)
    } else if state.pressed() {
        color_attribute(metadata, "focus_border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_border(tone))
    }
}

pub(super) fn alert_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

pub(super) fn alert_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "label_color"))
            .or_else(|| color_attribute(metadata, "mark_color"))
            .or_else(|| color_attribute(metadata, "status_mark_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

pub(super) fn alert_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_text_color(metadata, state, tone))
    }
}

fn alert_tone_surface<'a>(tone: AlertTone) -> Cow<'a, str> {
    let palette = feedback_palette();
    Cow::Borrowed(match tone {
        AlertTone::Info => &palette.alert_info_surface,
        AlertTone::Success => &palette.alert_success_surface,
        AlertTone::Warning => &palette.alert_warning_surface,
        AlertTone::Error => &palette.alert_error_surface,
    })
}

fn alert_tone_border<'a>(tone: AlertTone) -> Cow<'a, str> {
    let palette = feedback_palette();
    Cow::Borrowed(match tone {
        AlertTone::Info => &palette.alert_info_border,
        AlertTone::Success => &palette.alert_success_border,
        AlertTone::Warning => &palette.alert_warning_border,
        AlertTone::Error => &palette.alert_error_border,
    })
}

fn alert_tone_mark<'a>(tone: AlertTone) -> Cow<'a, str> {
    let palette = feedback_palette();
    Cow::Borrowed(match tone {
        AlertTone::Info => &palette.alert_info_mark,
        AlertTone::Success => &palette.alert_success_mark,
        AlertTone::Warning => &palette.alert_warning_mark,
        AlertTone::Error => &palette.alert_error_mark,
    })
}

pub(super) fn tooltip_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_surface)
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_surface))
    }
}

pub(super) fn tooltip_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_border)
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_border))
    }
}

pub(super) fn tooltip_title_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_title))
    }
}

pub(super) fn tooltip_body_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "body_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_body))
    }
}

pub(super) fn tooltip_icon_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "icon_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "icon_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_icon))
    }
}

pub(super) fn toast_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_surface)
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_surface_pressed))
    } else if state.pointer_hot() {
        color_attribute(metadata, "hover_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_surface_hover))
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_surface))
    }
}

pub(super) fn toast_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_border)
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_border))
    }
}

pub(super) fn toast_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_text))
    }
}

pub(super) fn toast_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "mark_color"))
            .or_else(|| color_attribute(metadata, "status_mark_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_action))
    }
}

pub(super) fn toast_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_action))
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
