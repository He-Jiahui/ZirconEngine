use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::ModelAsset;
use zircon_runtime::core::framework::render::{
    RenderMeshSnapshot, RenderVirtualGeometryDebugState,
};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::graphics::{
    VirtualGeometryGpuCompletion, VirtualGeometryRuntimeExtractOutput,
    VirtualGeometryRuntimeFeedback, VirtualGeometryRuntimePrepareInput,
    VirtualGeometryRuntimePrepareOutput, VirtualGeometryRuntimeProvider,
    VirtualGeometryRuntimeState as RuntimeStateContract, VirtualGeometryRuntimeStats,
    VirtualGeometryRuntimeUpdate,
};

use crate::virtual_geometry::VirtualGeometryRuntimeState;

#[derive(Clone, Debug, Default)]
pub struct PluginVirtualGeometryRuntimeProvider;

impl VirtualGeometryRuntimeProvider for PluginVirtualGeometryRuntimeProvider {
    fn create_state(&self) -> Box<dyn RuntimeStateContract> {
        Box::<PluginVirtualGeometryRuntimeState>::default()
    }

    fn build_extract_from_meshes(
        &self,
        meshes: &[RenderMeshSnapshot],
        debug: Option<RenderVirtualGeometryDebugState>,
        load_model: &mut dyn FnMut(ResourceId) -> Option<ModelAsset>,
    ) -> Option<VirtualGeometryRuntimeExtractOutput> {
        let output = crate::virtual_geometry::build_virtual_geometry_automatic_extract_from_meshes_with_debug(
            meshes,
            debug.unwrap_or_default(),
            |model_id| load_model(model_id),
        )?;
        Some(VirtualGeometryRuntimeExtractOutput::new(
            output.extract().clone(),
            output.cpu_reference_instances().to_vec(),
            output.bvh_visualization_instances().to_vec(),
        ))
    }
}

#[derive(Debug, Default)]
struct PluginVirtualGeometryRuntimeState {
    state: VirtualGeometryRuntimeState,
}

impl RuntimeStateContract for PluginVirtualGeometryRuntimeState {
    fn prepare_frame(
        &mut self,
        input: VirtualGeometryRuntimePrepareInput<'_>,
    ) -> VirtualGeometryRuntimePrepareOutput {
        let Some(extract) = input.extract() else {
            self.state = VirtualGeometryRuntimeState::default();
            return VirtualGeometryRuntimePrepareOutput::default();
        };
        self.state.register_extract(Some(extract));
        if let Some(plan) = input.page_upload_plan() {
            self.state.ingest_plan(input.generation(), plan);
        }
        let prepare = self.state.build_prepare_frame_with_segments(
            input.visible_clusters(),
            input.visibility_draw_segments(),
        );
        let evictable_page_ids = prepare
            .evictable_pages
            .iter()
            .map(|page| page.page_id)
            .collect();
        VirtualGeometryRuntimePrepareOutput::new(evictable_page_ids)
    }

    fn update_after_render(
        &mut self,
        feedback: VirtualGeometryRuntimeFeedback,
    ) -> VirtualGeometryRuntimeUpdate {
        let previous_slot_owners = self.state.resident_slot_owners();
        let previous_pending_pages = self.state.pending_page_ids();
        let confirmed_completion = feedback.gpu_completion().map(|completion| {
            confirmed_virtual_geometry_completion(
                completion,
                previous_slot_owners.iter().copied(),
                previous_pending_pages.iter().copied(),
            )
        });
        let completed_page_count = confirmed_completion
            .as_ref()
            .map(|completion| completion.completed_page_assignments().len())
            .unwrap_or(0);
        let replaced_page_count = confirmed_completion
            .as_ref()
            .map(|completion| completion.completed_page_replacements().len())
            .unwrap_or(0);

        if let Some(feedback) = feedback.visibility_feedback() {
            self.state.refresh_hot_resident_pages(feedback);
        }
        if let Some(completion) = confirmed_completion.as_ref() {
            self.state.complete_gpu_uploads_with_replacements(
                completion.completed_page_assignments().iter().copied(),
                completion.completed_page_replacements().iter().copied(),
                feedback.evictable_page_ids(),
            );
            self.state
                .apply_gpu_page_table_entries(completion.page_table_entries());
        } else if let Some(feedback) = feedback.visibility_feedback() {
            self.state.consume_feedback(feedback);
        }
        self.state.ingest_page_requests(
            feedback.generation(),
            feedback
                .node_and_cluster_cull_page_requests()
                .iter()
                .copied(),
        );
        let snapshot = self.state.snapshot();
        VirtualGeometryRuntimeUpdate::new(VirtualGeometryRuntimeStats::new(
            snapshot.page_table_entry_count(),
            snapshot.resident_page_count(),
            snapshot.pending_request_count(),
            snapshot.page_dependency_count(),
            completed_page_count,
            replaced_page_count,
        ))
    }
}

fn confirmed_virtual_geometry_completion(
    completion: &VirtualGeometryGpuCompletion,
    previous_slot_owners: impl IntoIterator<Item = (u32, u32)>,
    previous_pending_pages: impl IntoIterator<Item = u32>,
) -> VirtualGeometryGpuCompletion {
    let page_table_entries =
        crate::virtual_geometry::normalized_page_table_entries(completion.page_table_entries());
    let page_table_slot_by_page = page_table_entries
        .iter()
        .copied()
        .collect::<BTreeMap<u32, u32>>();
    let previous_pending_pages = previous_pending_pages.into_iter().collect::<BTreeSet<_>>();
    let final_resident_pages = page_table_slot_by_page
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let previous_page_by_slot = previous_slot_owners.into_iter().collect::<BTreeMap<_, _>>();
    let completed_page_assignments = page_table_entries
        .iter()
        .filter(|(page_id, _slot)| previous_pending_pages.contains(page_id))
        .copied()
        .collect::<Vec<_>>();
    let completed_page_replacements = page_table_entries
        .iter()
        .filter(|(page_id, _slot)| previous_pending_pages.contains(page_id))
        .filter_map(|(page_id, _reported_slot)| {
            let confirmed_slot = page_table_slot_by_page.get(page_id).copied()?;
            let previous_page_id = previous_page_by_slot.get(&confirmed_slot).copied()?;
            (previous_page_id != *page_id && !final_resident_pages.contains(&previous_page_id))
                .then_some((*page_id, previous_page_id))
        })
        .collect::<Vec<_>>();

    VirtualGeometryGpuCompletion::new(
        page_table_entries,
        completed_page_assignments,
        completed_page_replacements,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::{
        AssetUri, ModelPrimitiveAsset, VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset,
        VirtualGeometryClusterPageHeaderAsset, VirtualGeometryDebugMetadataAsset,
        VirtualGeometryHierarchyNodeAsset, VirtualGeometryPageDependencyAsset,
        VirtualGeometryRootClusterRangeAsset,
    };
    use zircon_runtime::core::math::{Transform, Vec3, Vec4};
    use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
    use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

    #[test]
    fn provider_builds_neutral_extract_output_from_cooked_model_meshes() {
        let provider = PluginVirtualGeometryRuntimeProvider;
        let model_id = ResourceId::from_stable_label("res://models/provider-vg.model.toml");
        let material_id = ResourceId::from_stable_label("builtin://material/default");
        let model = cooked_model_asset();
        let mesh = RenderMeshSnapshot {
            node_id: 44,
            transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            model: ResourceHandle::<ModelMarker>::new(model_id),
            material: ResourceHandle::<MaterialMarker>::new(material_id),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        };
        let mut load_model = |requested_id| (requested_id == model_id).then(|| model.clone());

        let output = provider
            .build_extract_from_meshes(
                &[mesh],
                Some(RenderVirtualGeometryDebugState {
                    forced_mip: Some(10),
                    visualize_bvh: true,
                    ..RenderVirtualGeometryDebugState::default()
                }),
                &mut load_model,
            )
            .expect("provider should build automatic VG output from cooked model data");

        assert_eq!(output.extract().instances.len(), 1);
        assert_eq!(output.extract().instances[0].source_model, Some(model_id));
        assert_eq!(output.extract().debug.forced_mip, Some(10));
        assert_eq!(output.extract().pages.len(), 1);
        assert_eq!(output.extract().page_dependencies.len(), 1);
        assert!(!output.cpu_reference_instances().is_empty());
        assert!(!output.bvh_visualization_instances().is_empty());
    }

    fn cooked_model_asset() -> ModelAsset {
        ModelAsset {
            uri: AssetUri::parse("res://models/provider-vg.model.toml")
                .expect("model uri should parse"),
            primitives: vec![ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                virtual_geometry: Some(VirtualGeometryAsset {
                    hierarchy_buffer: vec![VirtualGeometryHierarchyNodeAsset {
                        node_id: 0,
                        parent_node_id: None,
                        child_node_ids: Vec::new(),
                        cluster_start: 0,
                        cluster_count: 1,
                        page_id: 10,
                        mip_level: 10,
                        bounds_center: [0.0, 0.0, 0.0],
                        bounds_radius: 1.0,
                        screen_space_error: 0.25,
                    }],
                    cluster_headers: vec![VirtualGeometryClusterHeaderAsset {
                        cluster_id: 7,
                        page_id: 10,
                        hierarchy_node_id: 0,
                        lod_level: 10,
                        parent_cluster_id: None,
                        bounds_center: [0.0, 0.0, 0.0],
                        bounds_radius: 1.0,
                        screen_space_error: 0.25,
                    }],
                    cluster_page_headers: vec![VirtualGeometryClusterPageHeaderAsset {
                        page_id: 10,
                        start_offset: 0,
                        payload_size_bytes: 64,
                    }],
                    cluster_page_data: vec![vec![0; 64]],
                    root_page_table: vec![10],
                    page_dependencies: vec![VirtualGeometryPageDependencyAsset {
                        page_id: 10,
                        parent_page_id: None,
                        child_page_ids: Vec::new(),
                    }],
                    root_cluster_ranges: vec![VirtualGeometryRootClusterRangeAsset {
                        node_id: 0,
                        cluster_start: 0,
                        cluster_count: 1,
                    }],
                    debug: VirtualGeometryDebugMetadataAsset {
                        mesh_name: Some("ProviderCookedMesh".to_string()),
                        source_hint: Some("provider-test".to_string()),
                        notes: Vec::new(),
                    },
                }),
            }],
        }
    }
}
