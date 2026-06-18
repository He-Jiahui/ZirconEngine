use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_inspector_row_kind::{inspector_row_kind, InspectorRowKind};

mod disclosure;
mod primitives;
mod resource;
mod shadow;
mod style;

pub(super) fn push_inspector_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match inspector_row_kind(node) {
        Some(InspectorRowKind::Resource(resource)) => {
            resource::push_resource_row(commands, node, rect, clip, order, resource, opacity);
            true
        }
        Some(InspectorRowKind::Disclosure) => {
            disclosure::push_disclosure_row(commands, node, rect, clip, order, opacity);
            true
        }
        Some(InspectorRowKind::ShadowSelect) => {
            shadow::push_shadow_select_row(commands, node, rect, clip, order, opacity);
            true
        }
        Some(InspectorRowKind::ShadowCheck) => {
            shadow::push_shadow_check_row(commands, node, rect, clip, order, opacity);
            true
        }
        None => false,
    }
}

#[cfg(test)]
#[path = "template_inspector_rows_tests.rs"]
mod tests;
