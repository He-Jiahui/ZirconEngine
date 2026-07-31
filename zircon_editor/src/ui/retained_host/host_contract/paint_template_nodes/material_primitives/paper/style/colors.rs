use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::resolved_style_color;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    paper_background_color_from_host(node, current_host_palette())
}

fn paper_background_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(palette.popup)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_border_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
) -> Option<[u8; 4]> {
    paper_border_color_from_host(node, outlined, current_host_palette())
}

fn paper_border_color_from_host(
    node: &TemplatePaneNodeData,
    outlined: bool,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (paper_border_width(node, outlined) > 0.0).then_some(palette.border))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_border_width(
    node: &TemplatePaneNodeData,
    outlined: bool,
) -> f32 {
    paper_border_width_from_host(node, outlined, current_host_metrics())
}

fn paper_border_width_from_host(
    node: &TemplatePaneNodeData,
    outlined: bool,
    metrics: HostControlMetrics,
) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if outlined {
        configured.max(metrics.border_width)
    } else {
        configured
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn paper_background_and_outlined_border_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.popup = [10, 11, 12, 255];
        palette.border = [20, 21, 22, 255];

        let node = TemplatePaneNodeData::default();
        assert_eq!(
            paper_background_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );
        assert_eq!(paper_border_color_from_host(&node, false, palette), None);
        assert_eq!(
            paper_border_color_from_host(&node, true, palette),
            Some([20, 21, 22, 255])
        );
    }

    #[test]
    fn paper_declared_colors_override_palette_when_available() {
        let mut palette = PALETTE;
        palette.popup = [10, 11, 12, 255];
        palette.border = [20, 21, 22, 255];
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(40, 41, 42, 255)));

        assert_eq!(
            paper_background_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );
        assert_eq!(
            paper_border_color_from_host(&node, true, palette),
            Some([40, 41, 42, 255])
        );
    }

    #[test]
    fn paper_outlined_border_width_projects_from_shared_host_metrics() {
        let metrics = HostControlMetrics {
            border_width: 1.5,
            ..METRICS
        };

        assert_eq!(
            paper_border_width_from_host(&TemplatePaneNodeData::default(), true, metrics),
            1.5
        );
        assert_eq!(
            paper_border_width_from_host(&TemplatePaneNodeData::default(), false, metrics),
            0.0
        );
    }

    #[test]
    fn paper_declared_border_width_remains_authoritative() {
        let metrics = HostControlMetrics {
            border_width: 1.5,
            ..METRICS
        };
        let node = TemplatePaneNodeData {
            border_width: 2.5,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(paper_border_width_from_host(&node, true, metrics), 2.5);
        assert_eq!(paper_border_width_from_host(&node, false, metrics), 2.5);
    }
}
