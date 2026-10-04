use super::super::{RenderMaterialAlphaMode, RenderQueueValue};

/// 统一网格和精灵的作者队列语义：材质队列在透明度决定的基础队列上偏移。
/// 阶段输入的构造者应使用同一规则，避免跨几何类别的排序含义分叉。
pub(in crate::core::framework::render) fn resolved_phase_queue(
    alpha_mode: &RenderMaterialAlphaMode,
    render_queue: i32,
    material_queue: i32,
) -> RenderQueueValue {
    RenderQueueValue::from_authored_queue(alpha_mode, render_queue)
        .with_material_offset_i32(material_queue)
}
