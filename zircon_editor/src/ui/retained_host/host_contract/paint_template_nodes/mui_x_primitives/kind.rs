use super::super::super::data::TemplatePaneNodeData;
use super::{charts, chat, data_grid, pickers, tree_view};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum MuiXKind {
    TreeView,
    DataGrid,
    DateTimePickers,
    Chart(charts::ChartKind),
    Chat(chat::ChatKind),
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn mui_x_kind(
    node: &TemplatePaneNodeData,
) -> Option<MuiXKind> {
    let component_role = node.component_role.as_str();
    let role = node.role.as_str();
    if tree_view::is_tree_view(component_role, role) {
        Some(MuiXKind::TreeView)
    } else if data_grid::is_data_grid(component_role, role) {
        Some(MuiXKind::DataGrid)
    } else if pickers::is_date_time_picker(component_role, role) {
        Some(MuiXKind::DateTimePickers)
    } else if let Some(kind) = charts::chart_kind(component_role, role) {
        Some(MuiXKind::Chart(kind))
    } else {
        chat::chat_kind(component_role, role).map(MuiXKind::Chat)
    }
}
