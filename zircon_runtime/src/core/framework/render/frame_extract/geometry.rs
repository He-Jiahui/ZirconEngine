use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId};

use super::super::{
    build_mesh_phase_queue, CorePipelineKind, MaterialPropertyOverrideBlock, MeshPhaseInput,
    RenderComponentChangeArtifact, RenderLayerSet, RenderMaterialAlphaMode, RenderMeshSnapshot,
    RenderPhaseQueue, RenderPhaseQueueSummary, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract,
};
use super::resolved_phase_queue;

/// 场景生产端交给渲染阶段排序器的网格排序契约。
/// `mesh_index` 必须指向同一 `GeometryExtract` 的网格列表，排序队列按该索引回取。
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryPhaseInput {
    pub entity: EntityId,
    pub mesh_index: usize,
    pub material_alpha_mode: RenderMaterialAlphaMode,
    pub depth: f32,
    pub depth_bias: f32,
    pub render_queue: i32,
    pub material_queue: i32,
    pub order_in_layer: i32,
    pub ui_z_index: i32,
}

impl GeometryPhaseInput {
    pub fn new(
        entity: EntityId,
        mesh_index: usize,
        material_alpha_mode: RenderMaterialAlphaMode,
        depth: f32,
    ) -> Self {
        Self {
            entity,
            mesh_index,
            material_alpha_mode,
            depth,
            depth_bias: 0.0,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_render_queue(mut self, render_queue: i32) -> Self {
        self.render_queue = render_queue;
        self
    }

    pub const fn with_material_queue(mut self, material_queue: i32) -> Self {
        self.material_queue = material_queue;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }
}

/// 网格快照、阶段队列与增量变更的共同提交包。
/// `World` 生成它；可见性、资源投影和绘制准备分别消费这些侧带。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GeometryExtract {
    pub scene_changes: Option<Arc<RenderComponentChangeArtifact>>,
    pub meshes: Vec<RenderMeshSnapshot>,
    pub material_property_overrides: BTreeMap<EntityId, MaterialPropertyOverrideBlock>,
    pub phase_inputs: Vec<GeometryPhaseInput>,
    pub phase_queue: RenderPhaseQueue,
    pub static_batches: Vec<StaticMeshBatchExtract>,
    pub virtual_geometry: Option<RenderVirtualGeometryExtract>,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

// TODO: [CR-FRAME-0002] 核对静态批次的消费计划：当前仅测试与大小统计读取该字段；确认保留逐帧构建或接入绘制准备。
/// 可共享材质和网格资源的静态实例候选，保留源网格索引与实体归属。
/// 实体级材质覆盖不会进入此候选批次，以免合并不同绘制状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticMeshBatchExtract {
    pub model: ResourceHandle<ModelMarker>,
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    pub render_layer_mask: RenderLayerSet,
    pub mesh_indices: Vec<usize>,
    pub entities: Vec<EntityId>,
}

impl StaticMeshBatchExtract {
    pub fn instance_count(&self) -> usize {
        self.mesh_indices.len()
    }
}

impl GeometryExtract {
    /// 简化快照适配路径：没有场景作者提供的排序元数据时按不透明网格处理。
    /// 正式场景提取应调用携带阶段输入和材质覆盖的构造入口。
    pub fn from_meshes(core_pipeline: CorePipelineKind, meshes: Vec<RenderMeshSnapshot>) -> Self {
        let phase_inputs = meshes
            .iter()
            .enumerate()
            .map(|(mesh_index, mesh)| {
                GeometryPhaseInput::new(
                    mesh.node_id,
                    mesh_index,
                    RenderMaterialAlphaMode::Opaque,
                    mesh.transform.translation.z,
                )
            })
            .collect::<Vec<_>>();
        Self::from_meshes_and_phase_inputs(core_pipeline, meshes, phase_inputs)
    }

    pub fn from_meshes_and_phase_inputs(
        core_pipeline: CorePipelineKind,
        meshes: Vec<RenderMeshSnapshot>,
        phase_inputs: Vec<GeometryPhaseInput>,
    ) -> Self {
        Self::from_meshes_phase_inputs_and_overrides(
            core_pipeline,
            meshes,
            phase_inputs,
            BTreeMap::new(),
        )
    }

    pub fn from_meshes_phase_inputs_and_overrides(
        core_pipeline: CorePipelineKind,
        meshes: Vec<RenderMeshSnapshot>,
        phase_inputs: Vec<GeometryPhaseInput>,
        material_property_overrides: BTreeMap<EntityId, MaterialPropertyOverrideBlock>,
    ) -> Self {
        let phase_queue = build_mesh_phase_queue(
            core_pipeline,
            phase_inputs.iter().map(|input| MeshPhaseInput {
                entity: input.entity,
                mesh_index: input.mesh_index,
                queue: resolved_phase_queue(
                    &input.material_alpha_mode,
                    input.render_queue,
                    input.material_queue,
                ),
                depth: input.depth,
                depth_bias: input.depth_bias,
                camera_order: 0,
                sorting_layer: 0,
                order_in_layer: input.order_in_layer,
                y_sort: None,
                ui_z_index: input.ui_z_index,
            }),
        );

        let static_batches = build_static_mesh_batches(&meshes, &material_property_overrides);

        Self {
            scene_changes: None,
            meshes,
            material_property_overrides,
            phase_inputs,
            phase_queue,
            static_batches,
            virtual_geometry: None,
            virtual_geometry_debug: None,
        }
    }

    pub fn with_material_property_overrides(
        mut self,
        overrides: BTreeMap<EntityId, MaterialPropertyOverrideBlock>,
    ) -> Self {
        self.material_property_overrides = overrides;
        self.static_batches =
            build_static_mesh_batches(&self.meshes, &self.material_property_overrides);
        self
    }

    pub fn with_scene_changes(
        mut self,
        scene_changes: Option<Arc<RenderComponentChangeArtifact>>,
    ) -> Self {
        self.scene_changes = scene_changes;
        self
    }

    /// 切换核心管线后重建排序决策；调用方需保持 `phase_inputs` 与网格索引配对。
    pub fn rebuild_phase_queue(&mut self, core_pipeline: CorePipelineKind) {
        self.phase_queue = build_mesh_phase_queue(
            core_pipeline,
            self.phase_inputs.iter().map(|input| MeshPhaseInput {
                entity: input.entity,
                mesh_index: input.mesh_index,
                queue: resolved_phase_queue(
                    &input.material_alpha_mode,
                    input.render_queue,
                    input.material_queue,
                ),
                depth: input.depth,
                depth_bias: input.depth_bias,
                camera_order: 0,
                sorting_layer: 0,
                order_in_layer: input.order_in_layer,
                y_sort: None,
                ui_z_index: input.ui_z_index,
            }),
        );
    }

    /// Builds a diagnostics summary from the current sorted mesh phase queue.
    pub fn phase_queue_summary(&self) -> RenderPhaseQueueSummary {
        self.phase_queue.summary()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StaticMeshBatchKey<'a> {
    model: ResourceId,
    mesh: Option<ResourceId>,
    material: ResourceId,
    render_layers: &'a RenderLayerSet,
}

// 只提交可共享状态的静态实例候选；个体材质覆盖需保留独立绘制语义。
fn build_static_mesh_batches(
    meshes: &[RenderMeshSnapshot],
    material_property_overrides: &BTreeMap<EntityId, MaterialPropertyOverrideBlock>,
) -> Vec<StaticMeshBatchExtract> {
    let mut batch_indices_by_key: BTreeMap<StaticMeshBatchKey<'_>, Vec<usize>> = BTreeMap::new();
    for (mesh_index, mesh) in meshes.iter().enumerate() {
        if mesh.mobility != Mobility::Static {
            continue;
        }
        if material_property_overrides.contains_key(&mesh.node_id) {
            continue;
        }
        batch_indices_by_key
            .entry(StaticMeshBatchKey {
                model: mesh.model.id(),
                mesh: mesh.mesh.map(ResourceHandle::id),
                material: mesh.material.id(),
                render_layers: &mesh.common.layer_mask,
            })
            .or_default()
            .push(mesh_index);
    }

    batch_indices_by_key
        .into_values()
        .filter(|mesh_indices| mesh_indices.len() > 1)
        .map(|mesh_indices| {
            let first_mesh = &meshes[mesh_indices[0]];
            StaticMeshBatchExtract {
                model: first_mesh.model,
                mesh: first_mesh.mesh,
                material: first_mesh.material,
                render_layer_mask: first_mesh.common.layer_mask.clone(),
                entities: mesh_indices
                    .iter()
                    .map(|mesh_index| meshes[*mesh_index].node_id)
                    .collect(),
                mesh_indices,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
