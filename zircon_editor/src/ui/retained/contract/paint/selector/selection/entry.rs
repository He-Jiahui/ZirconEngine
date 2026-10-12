//! 模板选择控件的视觉入口；种类决定状态族与颜色配方，节点 checked/selected 单独保留持久选择语义。
//! 同一次选择共享一个主题投影，结果供绘制端使用，不执行切换状态或写回节点。

use super::super::super::resolved_state_for_node;
use super::super::colors::{control_accent, mark_label, selection_text, toggle_thumb};
use super::super::model::{WorkbenchSelectionControlKind, WorkbenchSelectionControlStyle};
use super::super::palette::workbench_selection_control_palette;
use super::super::state::family_for_kind;
use super::border::control_border;
use super::surface::control_surface;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

/// 为调用端识别的复选框、单选框或开关生成视觉结果；checked/selected 不会在此写回。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_selection_control_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
) -> WorkbenchSelectionControlStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(family_for_kind(kind));
    let checked = node.checked || node.selected;
    let palette = workbench_selection_control_palette();
    WorkbenchSelectionControlStyle {
        surface: control_surface(node, kind, state, checked, palette),
        border: control_border(node, kind, state, checked, palette),
        thumb: toggle_thumb(node, state, checked, palette),
        accent: control_accent(node, state, palette),
        text: selection_text(node, state, palette),
        label: mark_label(node, state, palette),
        state,
    }
}
