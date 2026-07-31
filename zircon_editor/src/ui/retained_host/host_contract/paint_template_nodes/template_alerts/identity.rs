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
    let key = key.to_ascii_lowercase();
    if key.contains("warning") {
        Some(AlertTone::Warning)
    } else if key.contains("error") || key.contains("danger") || key.contains("failed") {
        Some(AlertTone::Error)
    } else if key.contains("success") || key.contains("check") {
        Some(AlertTone::Success)
    } else if key.contains("info") {
        Some(AlertTone::Info)
    } else {
        None
    }
}
