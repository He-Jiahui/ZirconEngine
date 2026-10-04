//! Convert the side header projection into the physical band published by the host.
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::style::StyleDimension;

pub(super) fn effective_scale() -> f32 {
    let scale = crate::ui::retained_host::current_host_paint_scale_factor();
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

pub(super) fn physical_nodes(
    nodes: &ModelRc<ViewTemplateNodeData>,
    scale: f32,
) -> ModelRc<ViewTemplateNodeData> {
    // The side header cache already owns the projected physical result.
    model_rc(
        nodes
            .iter()
            .cloned()
            .map(|mut node| {
                for value in [
                    &mut node.frame.x,
                    &mut node.frame.y,
                    &mut node.frame.width,
                    &mut node.frame.height,
                    &mut node.font_size,
                    &mut node.corner_radius,
                    &mut node.border_width,
                    &mut node.button_style.element.border_width,
                    &mut node.button_style.element.corner_radius,
                ] {
                    *value *= scale;
                }
                for dimension in [
                    &mut node.button_style.width,
                    &mut node.button_style.height,
                    &mut node.button_style.element.width,
                    &mut node.button_style.element.height,
                ] {
                    if let StyleDimension::Fixed(value) = dimension {
                        *value *= scale;
                    }
                }
                node
            })
            .collect(),
    )
}
