//! 表头与表尾身份由模板 control_id 确定，供选择器和表格几何端使用；普通行不因相似名称自动获得特殊角色。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_table_header(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchTableHeader"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_table_tail(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchTableTail"
}
