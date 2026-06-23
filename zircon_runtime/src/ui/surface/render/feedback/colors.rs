use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use super::{color_attribute, state::FeedbackRenderState};

const TOOLTIP_SURFACE: &str = "#171c20";
const TOOLTIP_BORDER: &str = "#252d32";
const TOOLTIP_TITLE: &str = "#d0d9dd";
const TOOLTIP_BODY: &str = "#a8b3b8";
const TOOLTIP_ICON: &str = "#259ca7";
const ALERT_INFO_SURFACE: &str = "#122e48";
const ALERT_INFO_BORDER: &str = "#296596";
const ALERT_INFO_MARK: &str = "#35c7d0";
const ALERT_SUCCESS_SURFACE: &str = "#163927";
const ALERT_SUCCESS_BORDER: &str = "#357348";
const ALERT_SUCCESS_MARK: &str = "#42b883";
const ALERT_WARNING_SURFACE: &str = "#453214";
const ALERT_WARNING_BORDER: &str = "#845e23";
const ALERT_WARNING_MARK: &str = "#e0a33a";
const ALERT_ERROR_SURFACE: &str = "#482024";
const ALERT_ERROR_BORDER: &str = "#853d3a";
const ALERT_ERROR_MARK: &str = "#ef7066";
const TOAST_SURFACE: &str = "#153035";
const TOAST_SURFACE_HOVER: &str = "#183a3f";
const TOAST_SURFACE_PRESSED: &str = "#103c4a";
const TOAST_BORDER: &str = "#35c7d014";
const TOAST_TEXT: &str = "#cee0e2";
const TOAST_ACTION: &str = "#35c7d0";
const DISABLED_SURFACE: &str = "#252c31";
const DISABLED_BORDER: &str = "#343f47";
const DISABLED_TEXT: &str = "#59656c";
const FOCUS_BORDER: &str = "#35c7d0";

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
) -> &'a str {
    if state.unavailable() {
        DISABLED_SURFACE
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color")
            .unwrap_or_else(|| alert_tone_surface(tone))
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or_else(|| {
            color_attribute(metadata, "background_color").unwrap_or(alert_tone_surface(tone))
        })
    } else {
        color_attribute(metadata, "background_color").unwrap_or_else(|| alert_tone_surface(tone))
    }
}

pub(super) fn alert_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.unavailable() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or_else(|| alert_tone_border(tone))
    }
}

pub(super) fn alert_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

pub(super) fn alert_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "label_color"))
            .or_else(|| color_attribute(metadata, "mark_color"))
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

pub(super) fn alert_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .unwrap_or_else(|| alert_text_color(metadata, state, tone))
    }
}

fn alert_tone_surface(tone: AlertTone) -> &'static str {
    match tone {
        AlertTone::Info => ALERT_INFO_SURFACE,
        AlertTone::Success => ALERT_SUCCESS_SURFACE,
        AlertTone::Warning => ALERT_WARNING_SURFACE,
        AlertTone::Error => ALERT_ERROR_SURFACE,
    }
}

fn alert_tone_border(tone: AlertTone) -> &'static str {
    match tone {
        AlertTone::Info => ALERT_INFO_BORDER,
        AlertTone::Success => ALERT_SUCCESS_BORDER,
        AlertTone::Warning => ALERT_WARNING_BORDER,
        AlertTone::Error => ALERT_ERROR_BORDER,
    }
}

fn alert_tone_mark(tone: AlertTone) -> &'static str {
    match tone {
        AlertTone::Info => ALERT_INFO_MARK,
        AlertTone::Success => ALERT_SUCCESS_MARK,
        AlertTone::Warning => ALERT_WARNING_MARK,
        AlertTone::Error => ALERT_ERROR_MARK,
    }
}

pub(super) fn tooltip_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_SURFACE
    } else {
        color_attribute(metadata, "background_color").unwrap_or(TOOLTIP_SURFACE)
    }
}

pub(super) fn tooltip_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(TOOLTIP_BORDER)
    }
}

pub(super) fn tooltip_title_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TOOLTIP_TITLE)
    }
}

pub(super) fn tooltip_body_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "body_color"))
            .unwrap_or(TOOLTIP_BODY)
    }
}

pub(super) fn tooltip_icon_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "icon_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "icon_color").unwrap_or(TOOLTIP_ICON)
    }
}

pub(super) fn toast_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_SURFACE
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color").unwrap_or(TOAST_SURFACE_PRESSED)
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or(TOAST_SURFACE_HOVER)
    } else {
        color_attribute(metadata, "background_color").unwrap_or(TOAST_SURFACE)
    }
}

pub(super) fn toast_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_BORDER
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color").unwrap_or(FOCUS_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(TOAST_BORDER)
    }
}

pub(super) fn toast_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "foreground_color").unwrap_or(TOAST_TEXT)
    }
}

pub(super) fn toast_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "mark_color"))
            .unwrap_or(TOAST_ACTION)
    }
}

pub(super) fn toast_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> &'a str {
    if state.unavailable() {
        DISABLED_TEXT
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .unwrap_or(TOAST_ACTION)
    }
}
