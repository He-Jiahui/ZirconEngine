use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::select_workbench_list_row_style;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum ListRowAdornmentKind {
    Check,
    Chevron,
    DisabledDiamond,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_kind(
    node: &TemplatePaneNodeData,
) -> ListRowAdornmentKind {
    if is_unavailable_list_row_state(select_workbench_list_row_style(node).state) {
        ListRowAdornmentKind::DisabledDiamond
    } else if node.checked || node.selected {
        ListRowAdornmentKind::Check
    } else {
        ListRowAdornmentKind::Chevron
    }
}

fn is_unavailable_list_row_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
