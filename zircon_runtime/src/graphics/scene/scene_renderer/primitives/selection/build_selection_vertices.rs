use std::collections::HashMap;

use crate::core::framework::render::RenderMeshSnapshot;
use crate::core::framework::scene::EntityId;
use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

use super::super::line_geometry::{append_bounding_box_vertices, append_cross};

const INDEXED_SELECTION_LOOKUP_THRESHOLD: usize = 8;

/// 将选中模型的各局部 mesh 包围盒及选区锚点构成世界空间线条，交由 overlay 线条 pass 绘制。
/// 高亮描边受 outline_enabled 约束；锚点独立于高亮开关。
// TODO: [CR-SCENE-PRIM-0002] 直接 mesh 快照可绕过 model 绘制，但此路径只查询 streamer.model；需确认选中直接 mesh 时是否丢失包围盒。
pub(crate) fn build_selection_vertices(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    if let Some(highlights) = frame.overlays().highlights.as_ref() {
        if highlights.attributes().outline_enabled {
            let tint = highlights.attributes().tint_rgba;
            let color = Vec4::new(tint[0], tint[1], tint[2], tint[3]);
            visit_selected_meshes(frame.meshes(), highlights.entities(), |mesh_instance| {
                let Some(model) = streamer.model(&mesh_instance.model.id()) else {
                    return;
                };
                for mesh in &model.meshes {
                    append_bounding_box_vertices(
                        &mut vertices,
                        mesh.bounds_min,
                        mesh.bounds_max,
                        mesh_instance.transform.matrix(),
                        color,
                    );
                }
            });
        }
    }

    let camera = frame.effective_camera();
    for anchor in &frame.overlays().selection_anchors {
        append_cross(
            &mut vertices,
            anchor.position,
            anchor.size,
            anchor.color,
            camera.transform.right(),
            camera.transform.up(),
        );
    }

    vertices
}

/// 小选区线性查找，大选区建立索引；两条路径都保留选中列表顺序并只取每个实体首个快照。
fn visit_selected_meshes<'a>(
    meshes: &'a [RenderMeshSnapshot],
    entities: &[EntityId],
    mut visit: impl FnMut(&'a RenderMeshSnapshot),
) {
    if entities.len() <= INDEXED_SELECTION_LOOKUP_THRESHOLD {
        for owner in entities {
            if let Some(mesh) = meshes.iter().find(|mesh| mesh.node_id == *owner) {
                visit(mesh);
            }
        }
        return;
    }

    let mut meshes_by_owner = HashMap::with_capacity(meshes.len());
    for mesh in meshes {
        meshes_by_owner.entry(mesh.node_id).or_insert(mesh);
    }
    for owner in entities {
        if let Some(mesh) = meshes_by_owner.get(owner) {
            visit(mesh);
        }
    }
}

#[cfg(test)]
#[path = "tests/build_selection_vertices.rs"]
mod tests;
