use super::super::super::super::data::TemplatePaneNodeData;
use super::super::metrics::AxisLabelMetrics;
use super::super::style::scale_link_color;

const SCALE_LINK_CONNECTOR_BORDER_WIDTH: f32 = 0.0;
const SCALE_LINK_CONNECTOR_RADIUS: f32 = 0.0;

#[derive(Clone, Copy)]
pub(super) struct ScaleLinkQuadStyle {
    pub fill: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub(super) struct ScaleLinkCommandStyle {
    pub lobe: ScaleLinkQuadStyle,
    pub connector: ScaleLinkQuadStyle,
}

pub(super) fn scale_link_command_style(
    node: &TemplatePaneNodeData,
    metrics: &AxisLabelMetrics,
) -> ScaleLinkCommandStyle {
    let color = scale_link_color(node);
    ScaleLinkCommandStyle {
        lobe: ScaleLinkQuadStyle {
            fill: None,
            border: Some(color),
            border_width: metrics.link_connector_width,
            radius: metrics.link_lobe_radius,
        },
        connector: ScaleLinkQuadStyle {
            fill: Some(color),
            border: None,
            border_width: SCALE_LINK_CONNECTOR_BORDER_WIDTH,
            radius: SCALE_LINK_CONNECTOR_RADIUS,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> AxisLabelMetrics {
        AxisLabelMetrics {
            font_size: 11.0,
            line_height: 13.2,
            link_lobe_width: 6.0,
            link_lobe_height: 7.0,
            link_lobe_radius: 3.0,
            link_overlap: 2.0,
            link_connector_width: 1.0,
        }
    }

    #[test]
    fn scale_link_command_style_keeps_lobe_outline_and_filled_connector() {
        let style = scale_link_command_style(&TemplatePaneNodeData::default(), &metrics());

        assert_eq!(style.lobe.fill, None);
        assert!(style.lobe.border.is_some());
        assert_eq!(style.lobe.border_width, 1.0);
        assert_eq!(style.lobe.radius, 3.0);
        assert!(style.connector.fill.is_some());
        assert_eq!(style.connector.border, None);
        assert_eq!(style.connector.border_width, 0.0);
        assert_eq!(style.connector.radius, 0.0);
    }
}
