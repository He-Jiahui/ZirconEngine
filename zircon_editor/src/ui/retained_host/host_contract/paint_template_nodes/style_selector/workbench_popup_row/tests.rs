use super::model::WorkbenchPopupRowState;
use super::selection::select_workbench_popup_row_style;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn popup_row_selector_projects_full_semantic_state() {
    let open = select_workbench_popup_row_style(WorkbenchPopupRowState {
        open: true,
        ..WorkbenchPopupRowState::default()
    });
    assert_eq!(open.state, UiPainterResolvedState::Open);
    assert_eq!(open.background, Some(PALETTE.surface_hover));
    assert_eq!(open.text, PALETTE.text);

    let dragging = select_workbench_popup_row_style(WorkbenchPopupRowState {
        dragging: true,
        ..WorkbenchPopupRowState::default()
    });
    assert_eq!(dragging.state, UiPainterResolvedState::Dragging);
    assert_eq!(dragging.background, Some(PALETTE.surface_hover));
    assert_eq!(dragging.adornment, PALETTE.text);

    let drop_hovered = select_workbench_popup_row_style(WorkbenchPopupRowState {
        drop_hovered: true,
        ..WorkbenchPopupRowState::default()
    });
    assert_eq!(drop_hovered.state, UiPainterResolvedState::DropHovered);
    assert_eq!(drop_hovered.background, Some(PALETTE.surface_hover));
    assert_eq!(drop_hovered.shortcut, PALETTE.text_muted);
}

#[test]
fn popup_row_loading_state_uses_unavailable_visuals() {
    let loading_selected = select_workbench_popup_row_style(WorkbenchPopupRowState {
        loading: true,
        hovered: true,
        selected: true,
        danger: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(loading_selected.state, UiPainterResolvedState::Loading);
    assert_eq!(loading_selected.background, None);
    assert_eq!(loading_selected.selection_mark, None);
    assert_eq!(loading_selected.text, PALETTE.text_disabled);
    assert_eq!(loading_selected.shortcut, PALETTE.text_disabled);
    assert_eq!(loading_selected.adornment, PALETTE.text_disabled);
}
