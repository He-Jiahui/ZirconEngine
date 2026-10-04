use crate::core::framework::render::{
    build_mesh_phase_queue, GeometryPhaseInput, MeshPhaseInput, RenderMeshSnapshot,
    RenderPhaseMeshSource, RenderPhaseQueue, RenderQueueValue,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::mesh_draw::MeshCommandSortInput;
use super::material_draw_selection::MaterialDrawSelection;

#[derive(Clone, Copy)]
pub(super) struct PhaseOrderedMeshSnapshot<'a> {
    pub(super) snapshot: &'a RenderMeshSnapshot,
    pub(super) command_sort_input: MeshCommandSortInput,
}

/// 用当前选定材质的排序偏移修正视图 phase 顺序并过滤相机层；空队列或缺失单项输入沿用各自的旧排序回退。
pub(super) fn phase_ordered_meshes<'a>(
    frame: &'a ViewportRenderFrame,
    streamer: &ResourceStreamer,
    material_selection: &MaterialDrawSelection,
) -> Vec<PhaseOrderedMeshSnapshot<'a>> {
    phase_ordered_meshes_with_material_offsets(frame, |mesh| {
        material_sort_offsets(streamer, material_selection, mesh)
    })
}

fn phase_ordered_meshes_with_material_offsets<'a>(
    frame: &'a ViewportRenderFrame,
    material_sort_offsets: impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Vec<PhaseOrderedMeshSnapshot<'a>> {
    let camera_layers = frame.extract.view.selected_camera_layers();
    let phase_queue = &frame.extract.geometry.phase_queue;
    if phase_queue.items.is_empty() {
        return frame
            .meshes()
            .iter()
            .filter(|mesh| camera_layers.intersects(&mesh.common.layer_mask))
            .map(|mesh| PhaseOrderedMeshSnapshot {
                snapshot: mesh,
                command_sort_input: MeshCommandSortInput::new(
                    mesh.transform.translation.z,
                    mesh.node_id,
                ),
            })
            .collect();
    }

    let material_adjusted_phase_queue =
        material_adjusted_phase_queue(frame, &material_sort_offsets)
            .unwrap_or_else(|| frame.extract.geometry.phase_queue.clone());
    meshes_from_phase_queue(
        frame,
        &material_adjusted_phase_queue,
        &material_sort_offsets,
    )
}

fn meshes_from_phase_queue<'a>(
    frame: &'a ViewportRenderFrame,
    phase_queue: &RenderPhaseQueue,
    material_sort_offsets: &impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Vec<PhaseOrderedMeshSnapshot<'a>> {
    let camera_layers = frame.extract.view.selected_camera_layers();
    let mut phase_inputs_by_mesh_index = vec![None; frame.meshes().len()];
    for input in &frame.extract.geometry.phase_inputs {
        if let Some(slot) = phase_inputs_by_mesh_index.get_mut(input.mesh_index) {
            if slot.is_none() {
                *slot = Some(input);
            }
        }
    }
    phase_queue
        .items
        .iter()
        .filter_map(|item| match item.mesh_source {
            RenderPhaseMeshSource::MeshIndex(index) => {
                let snapshot = frame.meshes().get(index)?;
                if !camera_layers.intersects(&snapshot.common.layer_mask) {
                    return None;
                }
                let phase_input = phase_inputs_by_mesh_index
                    .get(index)
                    .and_then(|input| *input);
                let command_sort_input = command_sort_input_for_mesh_index(
                    frame,
                    index,
                    phase_input,
                    material_sort_offsets,
                )
                .unwrap_or_else(|| {
                    MeshCommandSortInput::new(snapshot.transform.translation.z, snapshot.node_id)
                });
                Some(PhaseOrderedMeshSnapshot {
                    snapshot,
                    command_sort_input,
                })
            }
            RenderPhaseMeshSource::SpriteIndex(_) => None,
        })
        .collect()
}

fn command_sort_input_for_mesh_index(
    frame: &ViewportRenderFrame,
    mesh_index: usize,
    input: Option<&GeometryPhaseInput>,
    material_sort_offsets: &impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Option<MeshCommandSortInput> {
    let input = input?;
    let mesh = frame.meshes().get(mesh_index)?;
    let offsets = material_sort_offsets(mesh);
    Some(MeshCommandSortInput {
        depth: input.depth,
        depth_bias: input.depth_bias + offsets.depth_bias,
        queue: material_adjusted_queue(
            &input.material_alpha_mode,
            input.render_queue,
            input.material_queue,
            offsets,
        ),
        camera_order: 0,
        sorting_layer: 0,
        order_in_layer: input.order_in_layer,
        y_sort: None,
        ui_z_index: input.ui_z_index,
        tie_breaker: input.entity,
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MaterialPhaseSortOffsets {
    queue: Option<RenderQueueValue>,
    fixed_queue: bool,
    render_queue: i32,
    material_queue: i32,
    depth_bias: f32,
}

fn material_sort_offsets(
    streamer: &ResourceStreamer,
    material_selection: &MaterialDrawSelection,
    mesh: &RenderMeshSnapshot,
) -> MaterialPhaseSortOffsets {
    material_selection
        .proxy(streamer, &mesh.material.id())
        .runtime()
        .map(|material| MaterialPhaseSortOffsets {
            queue: material.render_queue_value,
            fixed_queue: material.advanced_features.uses_transmission(),
            render_queue: material.render_queue,
            material_queue: material.material_queue,
            depth_bias: material.depth_bias,
        })
        .unwrap_or_default()
}

fn material_adjusted_phase_queue(
    frame: &ViewportRenderFrame,
    material_sort_offsets: &impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Option<RenderPhaseQueue> {
    let phase_inputs = frame.extract.geometry.phase_inputs.as_slice();
    (!phase_inputs.is_empty()).then(|| {
        build_mesh_phase_queue(
            frame.extract.view.core_pipeline,
            phase_inputs.iter().map(|input| {
                let offsets = frame
                    .meshes()
                    .get(input.mesh_index)
                    .map(|mesh| material_sort_offsets(mesh))
                    .unwrap_or_default();
                mesh_phase_input_with_material_offsets(input, offsets)
            }),
        )
    })
}

fn mesh_phase_input_with_material_offsets(
    input: &GeometryPhaseInput,
    offsets: MaterialPhaseSortOffsets,
) -> MeshPhaseInput {
    MeshPhaseInput {
        entity: input.entity,
        mesh_index: input.mesh_index,
        queue: material_adjusted_queue(
            &input.material_alpha_mode,
            input.render_queue,
            input.material_queue,
            offsets,
        ),
        depth: input.depth,
        depth_bias: input.depth_bias + offsets.depth_bias,
        camera_order: 0,
        sorting_layer: 0,
        order_in_layer: input.order_in_layer,
        y_sort: None,
        ui_z_index: input.ui_z_index,
    }
}

fn material_adjusted_queue(
    alpha_mode: &crate::core::framework::render::RenderMaterialAlphaMode,
    input_render_queue: i32,
    input_material_queue: i32,
    offsets: MaterialPhaseSortOffsets,
) -> RenderQueueValue {
    if offsets.fixed_queue {
        return offsets
            .queue
            .unwrap_or(crate::core::framework::render::STANDARD_PBR_TRANSMISSION_RENDER_QUEUE);
    }
    let queue = if let Some(queue) = offsets.queue {
        queue.with_material_offset_i32(input_render_queue)
    } else {
        RenderQueueValue::from_authored_queue(
            alpha_mode,
            input_render_queue.saturating_add(offsets.render_queue),
        )
    };
    queue.with_material_offset_i32(input_material_queue.saturating_add(offsets.material_queue))
}

#[cfg(test)]
#[path = "tests/phase_ordering.rs"]
mod tests;
