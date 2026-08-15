use super::super::super::style_selector::{
    select_workbench_status_signal_style, WorkbenchStatusSignalKind as StatusSignalKind,
    WorkbenchStatusSignalStyle,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

fn status_signal_style(
    node: &TemplatePaneNodeData,
    kind: StatusSignalKind,
) -> WorkbenchStatusSignalStyle {
    select_workbench_status_signal_style(node, kind)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_icon_fill(
    node: &TemplatePaneNodeData,
    kind: StatusSignalKind,
) -> [u8; 4] {
    status_signal_style(node, kind).icon_fill
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_text_color(
    node: &TemplatePaneNodeData,
    kind: StatusSignalKind,
) -> [u8; 4] {
    status_signal_style(node, kind).text
}
