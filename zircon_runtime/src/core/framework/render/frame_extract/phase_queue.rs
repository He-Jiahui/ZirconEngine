use super::super::{RenderMaterialAlphaMode, RenderQueueValue};

pub(in crate::core::framework::render) fn resolved_phase_queue(
    alpha_mode: &RenderMaterialAlphaMode,
    render_queue: i32,
    material_queue: i32,
) -> RenderQueueValue {
    RenderQueueValue::from_authored_queue(alpha_mode, render_queue)
        .with_material_offset_i32(material_queue)
}
