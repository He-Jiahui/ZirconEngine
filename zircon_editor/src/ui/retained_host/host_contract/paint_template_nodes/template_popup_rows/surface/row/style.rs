use super::super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::super::metrics::WorkbenchPopupRowMetrics;

pub(super) struct PopupRowSurfaceCommandStyle {
    pub fill: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_row_surface_command_style(
    style: WorkbenchPopupRowStyle,
    metrics: &WorkbenchPopupRowMetrics,
) -> Option<PopupRowSurfaceCommandStyle> {
    if style.background.is_none() && style.outline.is_none() {
        return None;
    }
    Some(PopupRowSurfaceCommandStyle {
        fill: style.background,
        border: style.outline,
        border_width: popup_row_surface_border_width(&style, metrics),
        radius: metrics.surface_radius,
    })
}

fn popup_row_surface_border_width(
    style: &WorkbenchPopupRowStyle,
    metrics: &WorkbenchPopupRowMetrics,
) -> f32 {
    if style.outline.is_some() {
        metrics.outline_width
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
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
            state: UiPainterResolvedState::Idle,
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
}
