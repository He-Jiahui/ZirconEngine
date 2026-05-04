use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::{
    merge_plugin_renderer_outputs, SceneRendererAdvancedPluginReadbacks,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        let collectors = self.runtime_prepare_collectors_mut();
        if collectors.is_empty() {
            return Ok(SceneRendererAdvancedPluginReadbacks::new());
        }

        let mut outputs = RenderPluginRendererOutputs::default();
        for collector in collectors {
            merge_plugin_renderer_outputs(
                &mut outputs,
                collector(device, queue, encoder, streamer, frame)?,
            );
        }

        Ok(SceneRendererAdvancedPluginReadbacks::from_outputs(outputs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderHybridGiReadbackOutputs,
        RenderOverlayExtract, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
        RenderPreparedRuntimeSidebands, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::{
        RuntimePrepareCollector, RuntimePrepareCollectorContext,
        RuntimePrepareCollectorRegistration,
    };
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    #[test]
    fn runtime_prepare_collectors_are_no_op_when_empty() {
        let (mut resources, device, queue, mut encoder, streamer, frame) =
            runtime_prepare_fixture();

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

    #[test]
    fn runtime_prepare_collectors_replace_overlapping_feature_packets() {
        let (mut resources, device, queue, mut encoder, streamer, frame) =
            runtime_prepare_fixture();
        resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _| {
            Ok(RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![1],
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![10],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 3,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
            })
        }));
        resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _| {
            Ok(RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![2, 3],
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![20],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 7,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
            })
        }));

        let readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        let outputs = readbacks.outputs_for_test();
        assert_eq!(outputs.virtual_geometry.page_table_entries, vec![2, 3]);
        assert_eq!(outputs.hybrid_gi.completed_probe_ids, vec![20]);
        assert_eq!(outputs.particles.alive_count, 7);
    }

    #[test]
    fn runtime_prepare_collectors_preserve_non_empty_packet_after_empty_packet() {
        let (mut resources, device, queue, mut encoder, streamer, frame) =
            runtime_prepare_fixture();
        resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _| {
            Ok(RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![5, 8],
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            })
        }));
        resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _| {
            Ok(RenderPluginRendererOutputs::default())
        }));

        let readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        assert_eq!(
            readbacks
                .outputs_for_test()
                .virtual_geometry
                .page_table_entries,
            vec![5, 8]
        );
    }

    #[test]
    fn runtime_prepare_collectors_can_mutate_per_frame_state() {
        let (mut resources, device, queue, mut encoder, streamer, frame) =
            runtime_prepare_fixture();
        let mut call_count = 0;
        resources.register_runtime_prepare_collector(Box::new(move |_, _, _, _, _| {
            call_count += 1;
            Ok(RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![call_count],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            })
        }));

        let first_readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();
        let second_readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        assert_eq!(
            first_readbacks
                .outputs_for_test()
                .hybrid_gi
                .completed_probe_ids,
            vec![1]
        );
        assert_eq!(
            second_readbacks
                .outputs_for_test()
                .hybrid_gi
                .completed_probe_ids,
            vec![2]
        );
    }

    #[test]
    fn registered_runtime_prepare_collector_can_read_frame_context_and_prepared_sidebands() {
        let (_resources, device, queue, mut encoder, streamer, frame) = runtime_prepare_fixture();
        let frame = frame.with_prepared_runtime_sidebands(RenderPreparedRuntimeSidebands::new(
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300, 301],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![7],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
            vec![11],
            vec![22],
        ));
        let called = Arc::new(AtomicBool::new(false));
        let mut resources = SceneRendererAdvancedPluginResources::new(
            &device,
            &[],
            [RuntimePrepareCollectorRegistration::new_collector(
                "test.context-sidebands",
                Arc::new(AssertingContextCollector {
                    called: Arc::clone(&called),
                }),
            )],
        );

        resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn runtime_prepare_collectors_can_project_prepared_sidebands_into_renderer_outputs() {
        let (mut resources, device, queue, mut encoder, streamer, frame) =
            runtime_prepare_fixture();
        let frame = frame.with_prepared_runtime_sidebands(RenderPreparedRuntimeSidebands::new(
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![501],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![77],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
            Vec::new(),
            Vec::new(),
        ));
        resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, frame| {
            Ok(frame
                .prepared_runtime_sidebands()
                .plugin_renderer_outputs
                .clone())
        }));

        let readbacks = resources
            .execute_runtime_prepare_passes(&device, &queue, &mut encoder, &streamer, &frame)
            .unwrap();

        let outputs = readbacks.outputs_for_test();
        assert_eq!(
            outputs.virtual_geometry.node_cluster_cull.page_request_ids,
            vec![501]
        );
        assert_eq!(outputs.hybrid_gi.completed_probe_ids, vec![77]);
    }

    struct AssertingContextCollector {
        called: Arc<AtomicBool>,
    }

    impl RuntimePrepareCollector for AssertingContextCollector {
        fn collect(
            &self,
            context: &mut RuntimePrepareCollectorContext<'_>,
        ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
            assert_eq!(context.viewport_size(), UVec2::new(1, 1));
            assert_eq!(context.prepared_hybrid_gi_evictable_probe_ids(), &[11]);
            assert_eq!(
                context
                    .prepared_virtual_geometry_readback_outputs()
                    .node_cluster_cull
                    .page_request_ids,
                vec![300, 301]
            );
            self.called.store(true, Ordering::SeqCst);
            Ok(RenderPluginRendererOutputs::default())
        }
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
            SceneRendererAdvancedPluginResources::new(&device, &[], Vec::new()),
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
