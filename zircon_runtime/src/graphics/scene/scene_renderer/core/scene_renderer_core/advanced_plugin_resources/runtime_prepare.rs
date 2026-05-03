use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        if self.runtime_prepare_collectors().is_empty()
            && !self.virtual_geometry_enabled()
            && !self.hybrid_gi_enabled()
        {
            return Ok(SceneRendererAdvancedPluginReadbacks::new());
        }

        let mut outputs = RenderPluginRendererOutputs::default();
        for collector in self.runtime_prepare_collectors() {
            merge_plugin_renderer_outputs(
                &mut outputs,
                collector(device, queue, encoder, streamer, frame)?,
            );
        }

        Ok(SceneRendererAdvancedPluginReadbacks::from_outputs(outputs))
    }
}

fn merge_plugin_renderer_outputs(
    target: &mut RenderPluginRendererOutputs,
    outputs: RenderPluginRendererOutputs,
) {
    target
        .virtual_geometry
        .page_table_entries
        .extend(outputs.virtual_geometry.page_table_entries);
    target
        .virtual_geometry
        .completed_page_assignments
        .extend(outputs.virtual_geometry.completed_page_assignments);
    target
        .virtual_geometry
        .page_replacements
        .extend(outputs.virtual_geometry.page_replacements);
    target
        .virtual_geometry
        .selected_clusters
        .extend(outputs.virtual_geometry.selected_clusters);
    target
        .virtual_geometry
        .visbuffer64_entries
        .extend(outputs.virtual_geometry.visbuffer64_entries);
    target
        .virtual_geometry
        .hardware_rasterization_records
        .extend(outputs.virtual_geometry.hardware_rasterization_records);
    target
        .virtual_geometry
        .node_cluster_cull
        .traversal_records
        .extend(outputs.virtual_geometry.node_cluster_cull.traversal_records);
    target
        .virtual_geometry
        .node_cluster_cull
        .child_work_items
        .extend(outputs.virtual_geometry.node_cluster_cull.child_work_items);
    target
        .virtual_geometry
        .node_cluster_cull
        .cluster_work_items
        .extend(
            outputs
                .virtual_geometry
                .node_cluster_cull
                .cluster_work_items,
        );
    target
        .virtual_geometry
        .node_cluster_cull
        .launch_worklist_snapshots
        .extend(
            outputs
                .virtual_geometry
                .node_cluster_cull
                .launch_worklist_snapshots,
        );
    target
        .virtual_geometry
        .node_cluster_cull
        .page_request_ids
        .extend(outputs.virtual_geometry.node_cluster_cull.page_request_ids);

    target
        .hybrid_gi
        .cache_entries
        .extend(outputs.hybrid_gi.cache_entries);
    target
        .hybrid_gi
        .completed_probe_ids
        .extend(outputs.hybrid_gi.completed_probe_ids);
    target
        .hybrid_gi
        .completed_trace_region_ids
        .extend(outputs.hybrid_gi.completed_trace_region_ids);
    target
        .hybrid_gi
        .probe_irradiance_rgb
        .extend(outputs.hybrid_gi.probe_irradiance_rgb);
    target
        .hybrid_gi
        .probe_rt_lighting_rgb
        .extend(outputs.hybrid_gi.probe_rt_lighting_rgb);
    merge_hybrid_gi_scene_prepare_outputs(
        &mut target.hybrid_gi.scene_prepare,
        outputs.hybrid_gi.scene_prepare,
    );

    target.particles.alive_count = target
        .particles
        .alive_count
        .saturating_add(outputs.particles.alive_count);
    target.particles.spawned_total = target
        .particles
        .spawned_total
        .saturating_add(outputs.particles.spawned_total);
    target.particles.debug_flags |= outputs.particles.debug_flags;
    target
        .particles
        .per_emitter_spawned
        .extend(outputs.particles.per_emitter_spawned);
    if outputs.particles.indirect_draw_args != [0; 4] {
        target.particles.indirect_draw_args = outputs.particles.indirect_draw_args;
    }
}

fn merge_hybrid_gi_scene_prepare_outputs(
    target: &mut crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs,
    outputs: crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs,
) {
    target
        .occupied_atlas_slots
        .extend(outputs.occupied_atlas_slots);
    target
        .occupied_capture_slots
        .extend(outputs.occupied_capture_slots);
    target.atlas_samples.extend(outputs.atlas_samples);
    target.capture_samples.extend(outputs.capture_samples);
    target.voxel_clipmap_ids.extend(outputs.voxel_clipmap_ids);
    target.voxel_samples.extend(outputs.voxel_samples);
    target.voxel_occupancy.extend(outputs.voxel_occupancy);
    target
        .voxel_occupancy_masks
        .extend(outputs.voxel_occupancy_masks);
    target.voxel_cells.extend(outputs.voxel_cells);
    target.voxel_cell_samples.extend(outputs.voxel_cell_samples);
    target
        .voxel_cell_dominant_nodes
        .extend(outputs.voxel_cell_dominant_nodes);
    target
        .voxel_cell_dominant_samples
        .extend(outputs.voxel_cell_dominant_samples);
    if outputs.texture_width != 0 {
        target.texture_width = outputs.texture_width;
    }
    if outputs.texture_height != 0 {
        target.texture_height = outputs.texture_height;
    }
    if outputs.texture_layers != 0 {
        target.texture_layers = outputs.texture_layers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderHybridGiReadbackOutputs,
        RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderVirtualGeometryReadbackOutputs, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::backend::RenderBackend;

    #[test]
    fn runtime_prepare_collectors_are_no_op_when_empty() {
        let (resources, device, queue, mut encoder, streamer, frame) = runtime_prepare_fixture();

        let readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        assert!(readbacks.is_empty());
        assert!(readbacks.outputs_for_test().is_empty());
    }

    #[test]
    fn runtime_prepare_collectors_return_neutral_plugin_renderer_outputs() {
        let (mut resources, device, queue, mut encoder, streamer, frame) =
            runtime_prepare_fixture();
        resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _| {
            Ok(RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![42, 43],
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![7],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            })
        }));

        let readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        assert!(!readbacks.is_empty());
        assert_eq!(
            readbacks
                .outputs_for_test()
                .virtual_geometry
                .page_table_entries,
            vec![42, 43]
        );
        assert_eq!(
            readbacks.outputs_for_test().hybrid_gi.completed_probe_ids,
            vec![7]
        );
    }

    fn runtime_prepare_fixture() -> (
        SceneRendererAdvancedPluginResources,
        wgpu::Device,
        wgpu::Queue,
        wgpu::CommandEncoder,
        ResourceStreamer,
        ViewportRenderFrame,
    ) {
        let backend = RenderBackend::new_offscreen().unwrap();
        let RenderBackend { device, queue, .. } = backend;
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-runtime-prepare-test-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let streamer = ResourceStreamer::new(
            std::sync::Arc::new(crate::asset::pipeline::manager::ProjectAssetManager::default()),
            &device,
            &queue,
            &texture_layout,
        );
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-prepare-test-encoder"),
        });
        let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(1, 1));

        (
            SceneRendererAdvancedPluginResources::new(&device, &[]),
            device,
            queue,
            encoder,
            streamer,
            frame,
        )
    }

    fn empty_scene_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }
}
