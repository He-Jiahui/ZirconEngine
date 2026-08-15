use std::borrow::Cow;
use std::sync::Arc;

use super::*;
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::graphics::backend::RenderBackend;

#[test]
fn collector_context_exposes_viewport_size_extract_and_prepared_sidebands() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(44),
        empty_scene_snapshot(),
    );
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720))
        .with_prepared_runtime_sidebands(RenderPreparedRuntimeSidebands::new(
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
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

    let mut external_buffer_bindings = Vec::new();
    let context = RuntimePrepareCollectorContext::new(
        &device,
        &queue,
        &mut encoder,
        &streamer,
        &frame,
        &mut external_buffer_bindings,
    );

    assert_eq!(context.viewport_size(), UVec2::new(1280, 720));
    assert_eq!(context.frame_extract().world.raw(), 44);
    assert_eq!(context.scene_snapshot().scene.meshes.len(), 0);
    assert_eq!(
        context
            .prepared_hybrid_gi_readback_outputs()
            .completed_probe_ids,
        vec![7]
    );
    assert_eq!(
        context
            .prepared_virtual_geometry_readback_outputs()
            .node_cluster_cull
            .page_request_ids,
        vec![300]
    );
    assert_eq!(context.prepared_hybrid_gi_evictable_probe_ids(), &[11]);
    assert_eq!(
        context.prepared_virtual_geometry_evictable_page_ids(),
        &[22]
    );
}

#[test]
fn collector_context_registers_external_buffer_bindings() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-buffer-binding-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(64, 64));
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-runtime-prepare-context-external-buffer"),
        size: 32,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut external_buffer_bindings = Vec::new();

    {
        let mut context = RuntimePrepareCollectorContext::new(
            &device,
            &queue,
            &mut encoder,
            &streamer,
            &frame,
            &mut external_buffer_bindings,
        );
        context.register_external_buffer_binding_with_backing(
            "particles.gpu.counters",
            "particles.gpu.counters:test-runtime-prepare",
            &buffer,
        );
        context.register_static_external_buffer_binding_with_backing(
            "particles.gpu.alive-indices",
            "particles.gpu.alive-indices:test-runtime-prepare",
            &buffer,
        );
    }

    assert_eq!(external_buffer_bindings.len(), 2);
    assert_eq!(
        external_buffer_bindings[0].logical_name(),
        "particles.gpu.counters"
    );
    assert_eq!(
        external_buffer_bindings[0].backing_name(),
        "particles.gpu.counters:test-runtime-prepare"
    );
    assert!(matches!(
        &external_buffer_bindings[0].logical_name,
        Cow::Owned(_)
    ));
    assert!(matches!(
        &external_buffer_bindings[0].backing_name,
        Cow::Owned(_)
    ));
    assert!(matches!(
        &external_buffer_bindings[1].logical_name,
        Cow::Borrowed("particles.gpu.alive-indices")
    ));
    assert!(matches!(
        &external_buffer_bindings[1].backing_name,
        Cow::Borrowed("particles.gpu.alive-indices:test-runtime-prepare")
    ));
}

#[test]
fn collector_context_returns_nonblocking_shared_queue_readback_handles() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-readback-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(1, 1));
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-runtime-prepare-context-readback-source"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let mut external_buffer_bindings = Vec::new();
    let mut gpu_readbacks = Vec::new();
    let mut context = RuntimePrepareCollectorContext::new_with_gpu_readbacks(
        &device,
        &queue,
        &mut encoder,
        &streamer,
        &frame,
        &mut external_buffer_bindings,
        &mut gpu_readbacks,
    );

    let readback = context
        .request_gpu_readback("test.runtime-prepare", &buffer, 0..4)
        .unwrap();

    assert!(!readback.is_ready());
    assert_eq!(gpu_readbacks.len(), 1);
    gpu_readbacks.pop().unwrap().fail("test readback rejection");
    assert!(readback.is_ready());
    assert!(readback.try_take().unwrap().is_err());
}

#[test]
fn collector_context_without_admission_rejects_new_gpu_readbacks() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-unadmitted-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(1, 1));
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-runtime-prepare-context-unadmitted-source"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let mut external_buffer_bindings = Vec::new();
    let mut gpu_readbacks = Vec::new();
    let mut gpu_pass_profiles = Vec::new();
    let mut context = RuntimePrepareCollectorContext::new_with_gpu_readbacks_and_gpu_work_admission(
        &device,
        &queue,
        &mut encoder,
        &streamer,
        &frame,
        &mut external_buffer_bindings,
        &mut gpu_readbacks,
        false,
        None,
        &mut gpu_pass_profiles,
    );

    assert!(!context.gpu_work_admitted());
    assert!(context
        .request_gpu_readback("test.runtime-prepare", &buffer, 0..4)
        .is_err());
    assert!(gpu_readbacks.is_empty());
    assert!(gpu_pass_profiles.is_empty());
}

#[test]
fn collector_context_retains_cpu_profile_for_an_admitted_gpu_pass() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-profile-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(1, 1));
    let mut external_buffer_bindings = Vec::new();
    let mut gpu_readbacks = Vec::new();
    let mut gpu_pass_profiles = Vec::new();

    {
        let mut context =
            RuntimePrepareCollectorContext::new_with_gpu_readbacks_and_gpu_work_admission(
                &device,
                &queue,
                &mut encoder,
                &streamer,
                &frame,
                &mut external_buffer_bindings,
                &mut gpu_readbacks,
                true,
                None,
                &mut gpu_pass_profiles,
            );
        let pass = context.begin_gpu_pass("runtime_prepare.test");
        context.end_gpu_pass(pass, "test.runtime-prepare", RenderBudgetKey::Other, 17);
    }

    assert_eq!(gpu_pass_profiles.len(), 1);
    assert_eq!(gpu_pass_profiles[0].pass_name, "runtime_prepare.test");
    assert_eq!(gpu_pass_profiles[0].executor_id, "test.runtime-prepare");
    assert_eq!(gpu_pass_profiles[0].budget_key, RenderBudgetKey::Other);
    assert_eq!(gpu_pass_profiles[0].cpu_elapsed_micros, 17);
}

#[test]
fn discarded_gpu_pass_does_not_publish_a_cpu_profile() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-discarded-profile-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(1, 1));
    let mut external_buffer_bindings = Vec::new();
    let mut gpu_readbacks = Vec::new();
    let mut gpu_pass_profiles = Vec::new();

    {
        let mut context =
            RuntimePrepareCollectorContext::new_with_gpu_readbacks_and_gpu_work_admission(
                &device,
                &queue,
                &mut encoder,
                &streamer,
                &frame,
                &mut external_buffer_bindings,
                &mut gpu_readbacks,
                true,
                None,
                &mut gpu_pass_profiles,
            );
        let pass = context.begin_gpu_pass("runtime_prepare.empty");
        context.discard_gpu_pass(pass);
    }

    assert!(gpu_pass_profiles.is_empty());
}

#[test]
fn collector_context_exposes_material_capture_streamer_accessors() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-material-capture-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let frame = ViewportRenderFrame::from_snapshot(empty_scene_snapshot(), UVec2::new(4, 4));
    let mut external_buffer_bindings = Vec::new();
    let context = RuntimePrepareCollectorContext::new(
        &device,
        &queue,
        &mut encoder,
        &streamer,
        &frame,
        &mut external_buffer_bindings,
    );

    let missing_material = ResourceId::from_stable_label("res://materials/missing.zmat");
    assert!(context.material_capture_seed(&missing_material).is_none());
    assert!(context.sample_texture_rgba(None, [0.5, 0.5]).is_none());
}

fn test_resource_streamer(device: &wgpu::Device, queue: &wgpu::Queue) -> ResourceStreamer {
    let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-runtime-prepare-context-test-texture-layout"),
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
    ResourceStreamer::new_for_test(
        Arc::new(crate::asset::pipeline::manager::ProjectAssetManager::default()),
        device,
        queue,
        &texture_layout,
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
