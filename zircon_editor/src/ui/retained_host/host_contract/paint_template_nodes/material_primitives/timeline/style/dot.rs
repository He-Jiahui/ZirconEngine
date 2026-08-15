use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::{component_variant_contains, resolved_style_color};
use super::tokens::timeline_dot_color_token;

const TIMELINE_DOT_BORDER_WIDTH: f32 = 2.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_dot_is_outlined(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "outlined")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_dot_background_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
    tone: [u8; 4],
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if outlined {
            None
        } else if timeline_dot_color_token(node) == "grey" {
            Some(tone)
        } else {
            Some(tone)
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_dot_border_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
    tone: [u8; 4],
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if outlined {
            Some(tone)
        } else {
            None
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_dot_border_width(
    node: &TemplatePaneNodeData,
    outlined: bool,
    has_border: bool,
) -> f32 {
    if !has_border {
        return 0.0;
    }
    let style_width = node.button_style.element.border_width;
    if style_width.is_finite() && style_width > 0.0 {
        style_width
    } else if node.border_width.is_finite() && node.border_width > 0.0 {
        node.border_width.max(if outlined {
            TIMELINE_DOT_BORDER_WIDTH
        } else {
            1.0
        })
    } else if outlined {
        TIMELINE_DOT_BORDER_WIDTH
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn timeline_grey_dot_uses_projected_tone_color() {
        let node = TemplatePaneNodeData::default();

        assert_eq!(
            timeline_dot_background_color(&node, false, [10, 11, 12, 255]),
            Some([10, 11, 12, 255])
        );
    }

    #[test]
    fn timeline_dot_declared_background_overrides_projected_tone() {
        let mut node = TemplatePaneNodeData::default();
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(20, 21, 22, 255)));

        assert_eq!(
            timeline_dot_background_color(&node, false, [10, 11, 12, 255]),
            Some([20, 21, 22, 255])
        );
    }
}
