use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};

pub(super) struct PopupBackgroundStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_background_style(node: &TemplatePaneNodeData) -> PopupBackgroundStyle {
    let palette = current_host_palette();
    let metrics = current_host_metrics();
    PopupBackgroundStyle {
        fill: palette.popup,
        border: palette.border,
        border_width: metrics.border_width,
        radius: if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
            node.corner_radius
        } else {
            metrics.radius_panel
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_background_uses_the_panel_radius_tier() {
        let metrics = current_host_metrics();
        let node = TemplatePaneNodeData::default();

        assert_eq!(popup_background_style(&node).radius, metrics.radius_panel);
    }

    #[test]
    fn popup_background_prefers_the_projected_panel_radius() {
        let mut node = TemplatePaneNodeData::default();
        node.corner_radius = 14.0;

        assert_eq!(popup_background_style(&node).radius, 14.0);
    }
}
