//! 控件局部绘制对集中选择器的窄入口；主题、焦点和交互状态在上游统一解析。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    select_workbench_selection_control_style, WorkbenchSelectionControlKind,
    WorkbenchSelectionControlStyle,
};

pub(super) fn selection_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
) -> WorkbenchSelectionControlStyle {
    select_workbench_selection_control_style(node, kind)
}
