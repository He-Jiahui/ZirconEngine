use super::super::super::data::TemplatePaneNodeData;
use super::DialogKind;

const DIALOG_SURFACE: [u8; 4] = [23, 28, 32, 255];
const DIALOG_BORDER: [u8; 4] = [52, 63, 71, 255];
const DIALOG_ACTIVE_BORDER: [u8; 4] = [53, 199, 208, 255];
const DIALOG_TITLE: [u8; 4] = [232, 236, 238, 255];
const DIALOG_BODY: [u8; 4] = [164, 174, 180, 255];
const DIALOG_ACTION: [u8; 4] = [53, 199, 208, 255];
const DIALOG_INFO: [u8; 4] = [53, 199, 208, 255];
const DIALOG_INFO_BORDER: [u8; 4] = [41, 101, 150, 255];
const DIALOG_WARNING: [u8; 4] = [224, 163, 58, 255];
const DIALOG_WARNING_BORDER: [u8; 4] = [132, 94, 35, 255];
const DIALOG_ERROR: [u8; 4] = [239, 112, 102, 255];
const DIALOG_ERROR_BORDER: [u8; 4] = [133, 61, 58, 255];
const DIALOG_DISABLED_SURFACE: [u8; 4] = [37, 44, 49, 255];
const DIALOG_DISABLED_BORDER: [u8; 4] = [52, 63, 71, 255];
const DIALOG_DISABLED_TEXT: [u8; 4] = [89, 101, 108, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogSeverity {
    Info,
    Warning,
    Error,
}

pub(super) fn dialog_unavailable(node: &TemplatePaneNodeData) -> bool {
    node.disabled || variant_contains_any(node, &["disabled", "loading"])
}

pub(super) fn dialog_surface_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_SURFACE
    } else {
        DIALOG_SURFACE
    }
}

pub(super) fn dialog_border_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_BORDER
    } else if matches!(kind, DialogKind::ConfirmDialog) {
        severity_border_color(node)
    } else if node.focused || node.pressed || node.popup_open {
        DIALOG_ACTIVE_BORDER
    } else {
        DIALOG_BORDER
    }
}

pub(super) fn dialog_title_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else if matches!(kind, DialogKind::ConfirmDialog)
        && (variant_contains_any(node, &["destructive"])
            || matches!(severity(node), DialogSeverity::Error))
    {
        severity_mark_color(node)
    } else {
        DIALOG_TITLE
    }
}

pub(super) fn dialog_body_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_BODY
    }
}

pub(super) fn dialog_action_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_ACTION
    }
}

pub(super) fn cancel_action_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_BODY
    }
}

pub(super) fn confirm_action_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    confirm_enabled: bool,
) -> [u8; 4] {
    if unavailable || !confirm_enabled {
        DIALOG_DISABLED_TEXT
    } else if variant_contains_any(node, &["destructive"]) {
        DIALOG_ERROR
    } else {
        DIALOG_ACTION
    }
}

pub(super) fn confirm_enabled(node: &TemplatePaneNodeData) -> bool {
    !variant_contains_any(
        node,
        &[
            "confirmDisabled",
            "confirm-disabled",
            "confirm_disabled",
            "disabledConfirm",
        ],
    )
}

fn severity(node: &TemplatePaneNodeData) -> DialogSeverity {
    if variant_contains_any(node, &["info"]) {
        DialogSeverity::Info
    } else if variant_contains_any(node, &["error", "danger"]) {
        DialogSeverity::Error
    } else {
        DialogSeverity::Warning
    }
}

pub(super) fn severity_mark_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match severity(node) {
        DialogSeverity::Info => DIALOG_INFO,
        DialogSeverity::Warning => DIALOG_WARNING,
        DialogSeverity::Error => DIALOG_ERROR,
    }
}

fn severity_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match severity(node) {
        DialogSeverity::Info => DIALOG_INFO_BORDER,
        DialogSeverity::Warning => DIALOG_WARNING_BORDER,
        DialogSeverity::Error => DIALOG_ERROR_BORDER,
    }
}

fn variant_contains_any(node: &TemplatePaneNodeData, expected: &[&str]) -> bool {
    [
        node.component_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.button_variant.as_str(),
    ]
    .iter()
    .flat_map(|value| value.split_whitespace())
    .any(|part| {
        expected
            .iter()
            .any(|expected| part.eq_ignore_ascii_case(expected))
    })
}
