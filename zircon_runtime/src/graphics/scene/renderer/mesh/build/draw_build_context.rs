use std::collections::HashSet;

use crate::graphics::types::ViewportRenderFrame;

use super::mesh_draw_build_context::MeshDrawBuildContext;

/// 从当前视口提取高亮实体与视图绕序，供同一轮 pending draw 收集使用。
/// 虚拟几何是否提交由后续间接绘制规划单独决定。
pub(super) fn build_mesh_draw_build_context(
    frame: &ViewportRenderFrame,
    _virtual_geometry_enabled: bool,
    reverse_view_raster_winding: bool,
) -> MeshDrawBuildContext {
    let selection = frame
        .overlays()
        .highlights
        .as_ref()
        .map(|highlights| highlights.entities().iter().copied())
        .into_iter()
        .flatten()
        .collect::<HashSet<_>>();

    MeshDrawBuildContext {
        selection,
        allowed_virtual_geometry_entities: None,
        reverse_view_raster_winding,
    }
}
