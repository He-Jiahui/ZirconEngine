use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::WorkbenchAlertTone as AlertTone;
use super::super::template_node_labels::template_node_label;

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
        "WorkbenchToastRoot" if is_standalone_toast(node) => Some(WorkbenchAlertKind::Toast),
        "WorkbenchToastRoot" => Some(WorkbenchAlertKind::Inline(
            alert_tone(node).unwrap_or(AlertTone::Info),
        )),
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

fn is_standalone_toast(node: &TemplatePaneNodeData) -> bool {
    let label = template_node_label(node, None).to_ascii_lowercase();
    label.contains("operation completed") || label.contains("completed successfully")
}

fn alert_tone(node: &TemplatePaneNodeData) -> Option<AlertTone> {
    let key = format!(
        "{} {} {} {} {} {}",
        node.control_id.as_str(),
        node.icon_name.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.component_variant.as_str(),
        template_node_label(node, None)
    )
    .to_ascii_lowercase();
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
