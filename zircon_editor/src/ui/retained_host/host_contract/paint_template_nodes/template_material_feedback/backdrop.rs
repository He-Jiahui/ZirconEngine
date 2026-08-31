use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;
use super::palette::material_feedback_palette;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_backdrop_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let variants = material_backdrop_variants(&node.component_variant);
    if !node.popup_open && node.surface_variant.as_str() != "backdrop" && !variants.open {
        return;
    }
    if variants.invisible {
        return;
    }
    if intersect(rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(
            resolved_style_color(node.button_style.element.background_color.as_ref())
                .unwrap_or_else(|| material_feedback_palette().backdrop_scrim),
        ),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MaterialBackdropVariants {
    open: bool,
    invisible: bool,
}

fn material_backdrop_variants(component_variant: &str) -> MaterialBackdropVariants {
    let mut variants = MaterialBackdropVariants::default();
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        variants.open |= part.eq_ignore_ascii_case("open");
        variants.invisible |= part.eq_ignore_ascii_case("invisible");
    }
    variants
}

#[cfg(test)]
#[path = "backdrop/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_material_backdrop_node(
    node: &TemplatePaneNodeData,
) -> bool {
    node.component_role.as_str() == "backdrop"
        || node.role.as_str() == "Backdrop"
        || node.surface_variant.as_str() == "backdrop"
}
