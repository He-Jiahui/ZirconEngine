use super::super::super::super::metrics::workbench_popup_row_metrics;
use super::*;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

fn row_style(background: Option<[u8; 4]>, outline: Option<[u8; 4]>) -> WorkbenchPopupRowStyle {
    WorkbenchPopupRowStyle {
        background,
        outline,
        text: [255; 4],
        shortcut: [255; 4],
        adornment: [255; 4],
        state: UiPainterResolvedState::Normal,
    }
}

#[test]
fn focus_outline_survives_a_transparent_row_background() {
    let metrics = workbench_popup_row_metrics();
    let style =
        popup_row_surface_command_style(row_style(None, Some([18, 180, 170, 255])), &metrics)
            .expect("an outline is independently drawable");

    assert_eq!(style.fill, None);
    assert_eq!(style.border, Some([18, 180, 170, 255]));
    assert_eq!(style.border_width, metrics.outline_width);
}

#[test]
fn idle_transparent_row_does_not_emit_a_surface() {
    let metrics = workbench_popup_row_metrics();

    assert!(popup_row_surface_command_style(row_style(None, None), &metrics).is_none());
}
