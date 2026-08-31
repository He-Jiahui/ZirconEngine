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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        render_mesh_stable_instance_key, RenderMaterialPropertyValue, RenderMeshStaticState,
    };
    use crate::core::math::{Transform, Vec4};

    #[test]
    fn geometry_extract_excludes_material_override_entities_from_static_batches() {
        let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "material:override-batch",
        ));
        let meshes = vec![test_static_mesh(1, material), test_static_mesh(2, material)];
        let geometry = GeometryExtract::from_meshes(CorePipelineKind::Core3d, meshes);
        assert_eq!(geometry.static_batches.len(), 1);

        let overrides = BTreeMap::from([(
            1,
            MaterialPropertyOverrideBlock::new()
                .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.0 }),
        )]);
        let geometry = geometry.with_material_property_overrides(overrides);

        assert!(geometry.static_batches.is_empty());
    }

    #[test]
    fn geometry_extract_builds_static_batches_against_supplied_overrides_once() {
        let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "material:constructor-override-batch",
        ));
        let meshes = vec![test_static_mesh(1, material), test_static_mesh(2, material)];
        let overrides = BTreeMap::from([(
            1,
            MaterialPropertyOverrideBlock::new()
                .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.0 }),
        )]);

        let geometry = GeometryExtract::from_meshes_phase_inputs_and_overrides(
            CorePipelineKind::Core3d,
            meshes,
            Vec::new(),
            overrides,
        );

        assert!(geometry.static_batches.is_empty());
        assert_eq!(geometry.material_property_overrides.len(), 1);
    }

    #[test]
    fn static_batch_key_borrows_render_layers_without_per_mesh_projection() {
        let source = include_str!("geometry.rs");

        assert!(source.contains(concat!("render_layers: &'a", " RenderLayerSet")));
        assert!(!source.contains(concat!("render_layers:", " Vec<u32>")));
        assert!(!source.contains(concat!(
            "render_layers: mesh.common.layer_mask",
            ".iter().collect()"
        )));
    }

    fn test_static_mesh(
        node_id: EntityId,
        material: ResourceHandle<MaterialMarker>,
    ) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
            transform_revision: 1,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "model:override-batch",
            )),
            mesh: None,
            material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Static,
            static_state: RenderMeshStaticState::new(true, 1, 1),
            common: crate::core::framework::render::RendererCommon {
                layer_mask: RenderLayerSet::default(),
                is_static: true,
                ..Default::default()
            },
        }
    }
}
