use super::super::resolved_state_for_node;
use super::colors::apply_declared_alert_colors;
use super::model::{WorkbenchAlertStyle, WorkbenchAlertTone};
use super::state::alert_state_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_alert_style(
    node: &TemplatePaneNodeData,
    tone: WorkbenchAlertTone,
) -> WorkbenchAlertStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Alert);
    let mut style = alert_state_style(tone, state);

    apply_declared_alert_colors(node, &mut style);

    style
}
