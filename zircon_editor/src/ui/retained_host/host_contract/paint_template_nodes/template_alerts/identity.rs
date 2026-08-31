use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::WorkbenchAlertTone as AlertTone;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchAlertKind {
    Inline(AlertTone),
    Toast,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_alert_kind(
    node: &TemplatePaneNodeData,
) -> Option<WorkbenchAlertKind> {
    match node.control_id.as_str() {
        "WorkbenchInfoAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Info)),
        "WorkbenchSuccessAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Success)),
        "WorkbenchWarningAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Warning)),
        "WorkbenchErrorAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Error)),
        "WorkbenchToastRoot" => Some(WorkbenchAlertKind::Toast),
        _ if node.control_id.as_str().starts_with("Workbench")
            && (matches!(node.role.as_str(), "Alert")
                || matches!(node.component_role.as_str(), "alert" | "mui-alert")
                || node.control_id.as_str().ends_with("Alert")) =>
        {
            alert_tone(node).map(WorkbenchAlertKind::Inline)
        }
        _ => None,
    }
}

fn alert_tone(node: &TemplatePaneNodeData) -> Option<AlertTone> {
    tone_from_key(node.validation_level.as_str())
        .or_else(|| tone_from_key(node.component_variant.as_str()))
        .or_else(|| tone_from_key(node.icon_name.as_str()))
        .or_else(|| tone_from_key(node.text_tone.as_str()))
}

fn tone_from_key(key: &str) -> Option<AlertTone> {
    if contains_ignore_ascii_case(key, "warning") {
        Some(AlertTone::Warning)
    } else if contains_ignore_ascii_case(key, "error")
        || contains_ignore_ascii_case(key, "danger")
        || contains_ignore_ascii_case(key, "failed")
    {
        Some(AlertTone::Error)
    } else if contains_ignore_ascii_case(key, "success") || contains_ignore_ascii_case(key, "check")
    {
        Some(AlertTone::Success)
    } else if contains_ignore_ascii_case(key, "info") {
        Some(AlertTone::Info)
    } else {
        None
    }
}

fn contains_ignore_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
