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
    RuntimePrepareCollector, RuntimePrepareCollectorContext, RuntimePrepareCollectorRegistration,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[test]
fn runtime_prepare_collectors_are_no_op_when_empty() {
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();

    let readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();

    assert!(readbacks.is_empty());
    assert!(readbacks.outputs_for_test().is_empty());
}

#[test]
fn runtime_prepare_collectors_return_neutral_plugin_renderer_outputs() {
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
    resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _, _, _, _, _, _, _| {
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
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
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
fn runtime_prepare_collectors_merge_overlapping_feature_packets() {
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
    resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _, _, _, _, _, _, _| {
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
    resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _, _, _, _, _, _, _| {
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
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();

    let outputs = readbacks.outputs_for_test();
    assert_eq!(outputs.virtual_geometry.page_table_entries, vec![1, 2, 3]);
    assert_eq!(outputs.hybrid_gi.completed_probe_ids, vec![10, 20]);
    assert_eq!(outputs.particles.alive_count, 7);
}

#[test]
fn runtime_prepare_collectors_preserve_non_empty_packet_after_empty_packet() {
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
    resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _, _, _, _, _, _, _| {
        Ok(RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![5, 8],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        })
    }));
    resources.register_runtime_prepare_collector(Box::new(|_, _, _, _, _, _, _, _, _, _, _| {
        Ok(RenderPluginRendererOutputs::default())
    }));

    let readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
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
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
    let mut call_count = 0;
    resources.register_runtime_prepare_collector(Box::new(
        move |_, _, _, _, _, _, _, _, _, _, _| {
            call_count += 1;
            Ok(RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![call_count],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            })
        },
    ));

    let first_readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();
    let second_readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
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
    let (_resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
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
        &[],
        [RuntimePrepareCollectorRegistration::new_collector(
            "test.context-sidebands",
            Arc::new(AssertingContextCollector {
                called: Arc::clone(&called),
                expected_device_epoch: RuntimePrepareDeviceEpoch::from_device_profile(
                    &device_profile,
                ),
            }),
        )],
    );

    resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();

    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn runtime_prepare_buffer_uploads_leave_collectors_as_one_frame_transaction() {
    let (_resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-runtime-prepare-upload-transaction-test"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });
    let mut resources = SceneRendererAdvancedPluginResources::new(
        &[],
        [RuntimePrepareCollectorRegistration::new_collector(
            "test.buffer-upload-transaction",
            Arc::new(RecordingBufferUploadCollector { buffer }),
        )],
    );

    let mut readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();

    assert!(!readbacks.take_runtime_prepare_buffer_uploads().is_empty());
    assert!(readbacks.take_runtime_prepare_buffer_uploads().is_empty());
}

#[test]
fn runtime_prepare_plumbing_does_not_expose_raw_queue_authority() {
    let sources = [
        include_str!("../../../../../../runtime_prepare_collector.rs"),
        include_str!("../scene_renderer_advanced_plugin_resources.rs"),
        include_str!("../runtime_prepare.rs"),
        include_str!(
            "../../../scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs"
        ),
    ];

    for source in sources {
        let production = source.split("\n#[cfg(test)]").next().unwrap_or_default();
        assert!(!production.contains("wgpu::Queue"));
        assert!(!production.contains("pub queue"));
    }
}

#[test]
fn runtime_prepare_collectors_can_project_prepared_sidebands_into_renderer_outputs() {
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
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
    resources.register_runtime_prepare_collector(Box::new(
        |_, _, _, _, frame, _, _, _, _, _, _| {
            Ok(frame
                .prepared_runtime_sidebands()
                .plugin_renderer_outputs
                .clone())
        },
    ));

    let readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();

    let outputs = readbacks.outputs_for_test();
    assert_eq!(
        outputs.virtual_geometry.node_cluster_cull.page_request_ids,
        vec![501]
    );
    assert_eq!(outputs.hybrid_gi.completed_probe_ids, vec![77]);
}

#[test]
fn runtime_prepare_collectors_can_register_external_buffer_bindings() {
    let (mut resources, device_profile, device, mut encoder, streamer, frame) =
        runtime_prepare_fixture();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-runtime-prepare-collector-plugin-buffer"),
        size: 64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    resources.register_runtime_prepare_collector(Box::new(
        move |device,
              device_epoch,
              encoder,
              streamer,
              frame,
              external_buffer_bindings,
              _,
              _,
              _,
              _,
              _| {
            let mut context = RuntimePrepareCollectorContext::new(
                device,
                device_epoch,
                encoder,
                streamer,
                frame,
                external_buffer_bindings,
            );
            context.register_external_buffer_binding_with_backing(
                "particles.gpu.counters",
                "particles.gpu.counters:test-runtime-prepare",
                &buffer,
            );
            Ok(RenderPluginRendererOutputs::default())
        },
    ));

    let readbacks = resources
        .execute_runtime_prepare_passes(&device_profile, &device, &mut encoder, &streamer, &frame)
        .unwrap();

    let bindings = readbacks
        .external_buffer_binding_packet()
        .expect("registered bindings must retain their device-qualified packet")
        .bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].logical_name(), "particles.gpu.counters");
    assert_eq!(
        bindings[0].backing_name(),
        "particles.gpu.counters:test-runtime-prepare"
    );
    assert!(readbacks.outputs_for_test().is_empty());
}

struct AssertingContextCollector {
    called: Arc<AtomicBool>,
    expected_device_epoch: RuntimePrepareDeviceEpoch,
}

struct RecordingBufferUploadCollector {
    buffer: wgpu::Buffer,
}

impl RuntimePrepareCollector for RecordingBufferUploadCollector {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        context
            .buffer_upload_recorder()
            .write_buffer(&self.buffer, 0, &[1, 2, 3, 4]);
        Ok(RenderPluginRendererOutputs::default())
    }
}

impl RuntimePrepareCollector for AssertingContextCollector {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        assert_eq!(context.device_epoch(), self.expected_device_epoch);
        assert_eq!(context.viewport_size(), UVec2::new(1, 1));
        assert_eq!(context.prepared_hybrid_gi_evictable_probe_ids(), &[11]);
        assert_eq!(
            context
                .prepared_virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300, 301]
        );
        let missing_material = crate::core::resource::ResourceId::from_stable_label(
            "res://materials/runtime-prepare-missing.zmat",
        );
        assert!(context.material_capture_seed(&missing_material).is_none());
        assert!(context.sample_texture_rgba(None, [0.5, 0.5]).is_none());
        self.called.store(true, Ordering::SeqCst);
        Ok(RenderPluginRendererOutputs::default())
    }
}

fn runtime_prepare_fixture() -> (
    SceneRendererAdvancedPluginResources,
    RenderDeviceProfile,
    wgpu::Device,
    wgpu::CommandEncoder,
    ResourceStreamer,
    ViewportRenderFrame,
) {
    let backend = RenderBackend::new_offscreen().unwrap();
    let device_profile = backend.device_profile().clone();
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
    let streamer = ResourceStreamer::new_for_test(
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
        SceneRendererAdvancedPluginResources::new(&[], Vec::new()),
        device_profile,
        device,
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
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment: crate::core::framework::render::EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}
