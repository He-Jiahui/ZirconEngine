use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::{
    WorkbenchSelectionControlKind, WorkbenchSelectionControlStyle,
    select_workbench_selection_control_style,
};

pub(super) fn selection_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
) -> WorkbenchSelectionControlStyle {
    select_workbench_selection_control_style(node, kind)
}
