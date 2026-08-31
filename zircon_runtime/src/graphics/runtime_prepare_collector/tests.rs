use std::borrow::Cow;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use crate::graphics::backend::RenderBackend;
use crate::rhi::{BufferDesc, BufferUsage, DeviceGeneration, DeviceId};

struct DeclaredGpuReadbackCollector;

impl RuntimePrepareCollector for DeclaredGpuReadbackCollector {
    fn requests_gpu_readback(&self) -> bool {
        true
    }

    fn collect(
        &self,
        _context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        Ok(RenderPluginRendererOutputs::default())
    }
}

fn empty_runtime_prepare_collector(
    _context: &mut RuntimePrepareCollectorContext<'_>,
) -> Result<RenderPluginRendererOutputs, GraphicsError> {
    Ok(RenderPluginRendererOutputs::default())
}

fn test_device_epoch() -> RuntimePrepareDeviceEpoch {
    RuntimePrepareDeviceEpoch::new(DeviceId::new(1), DeviceGeneration::initial())
}

#[test]
fn collector_root_keeps_native_gpu_authority_private_and_scoped() {
    let source = include_str!("../runtime_prepare_collector.rs");
    let owner = source
        .split("pub struct RuntimePrepareCollectorContext")
        .nth(1)
        .and_then(|source| source.split("/// Runtime-prepare capability").next())
        .expect("runtime prepare collector root owner");
    let gpu_capability = source
        .split("pub struct RuntimePrepareGpuRecordingContext")
        .nth(1)
        .and_then(|source| source.split("/// Recorder for CPU state").next())
        .expect("scoped runtime prepare GPU capability");

    assert!(!owner.contains("pub device:"));
    assert!(!owner.contains("pub encoder:"));
    assert!(!owner.contains("pub frame_extract:"));
    assert!(gpu_capability.contains("pub device:"));
    assert!(gpu_capability.contains("pub device_epoch:"));
    assert!(gpu_capability.contains("pub encoder:"));
    assert!(gpu_capability.contains("pub buffer_uploads:"));
    assert!(gpu_capability.contains("pub frame_transactions:"));
}

#[test]
fn runtime_prepare_device_epoch_preserves_typed_device_identity() {
    let epoch = RuntimePrepareDeviceEpoch::new(DeviceId::new(7), DeviceGeneration::new(11));

    assert_eq!(epoch.device_id(), DeviceId::new(7));
    assert_eq!(epoch.generation(), DeviceGeneration::new(11));
    assert_eq!(epoch, epoch);
    assert_ne!(
        epoch,
        RuntimePrepareDeviceEpoch::new(DeviceId::new(7), DeviceGeneration::new(12))
    );
}

#[test]
fn runtime_prepare_frame_transaction_commits_without_rollback() {
    let commits = Arc::new(AtomicUsize::new(0));
    let rollbacks = Arc::new(AtomicUsize::new(0));
    let transaction = RuntimePrepareFrameTransaction::new(
        "tests.runtime-prepare-commit",
        {
            let commits = Arc::clone(&commits);
            move || {
                commits.fetch_add(1, Ordering::SeqCst);
            }
        },
        {
            let rollbacks = Arc::clone(&rollbacks);
            move || {
                rollbacks.fetch_add(1, Ordering::SeqCst);
            }
        },
    );

    transaction.commit();

    assert_eq!(commits.load(Ordering::SeqCst), 1);
    assert_eq!(rollbacks.load(Ordering::SeqCst), 0);
}

#[test]
fn dropped_runtime_prepare_frame_transaction_rolls_back_once() {
    let commits = Arc::new(AtomicUsize::new(0));
    let rollbacks = Arc::new(AtomicUsize::new(0));
    let transaction = RuntimePrepareFrameTransaction::new(
        "tests.runtime-prepare-rollback",
        {
            let commits = Arc::clone(&commits);
            move || {
                commits.fetch_add(1, Ordering::SeqCst);
            }
        },
        {
            let rollbacks = Arc::clone(&rollbacks);
            move || {
                rollbacks.fetch_add(1, Ordering::SeqCst);
            }
        },
    );

    drop(transaction);

    assert_eq!(commits.load(Ordering::SeqCst), 0);
    assert_eq!(rollbacks.load(Ordering::SeqCst), 1);
}

#[test]
fn collector_registration_exposes_only_explicit_gpu_readback_requirements() {
    let declared = RuntimePrepareCollectorRegistration::new_collector(
        "tests.declared-readback",
        Arc::new(DeclaredGpuReadbackCollector),
    );
    let defaulted = RuntimePrepareCollectorRegistration::new(
        "tests.default-no-readback",
        empty_runtime_prepare_collector,
    );

    assert!(declared.requests_gpu_readback());
    assert!(!defaulted.requests_gpu_readback());
}

#[test]
fn runtime_prepare_readbacks_use_the_product_diagnostic_router_without_byte_cloning() {
    let source = include_str!("../runtime_prepare_collector.rs");
    let readback = include_str!("../runtime_prepare_collector/gpu_readback.rs");

    assert!(source.contains("DEFAULT_RUNTIME_PREPARE_MAX_IN_FLIGHT_READBACK_FRAMES: usize = 3"));
    assert!(source.contains("self.device_epoch"));
    assert!(readback.contains("device_epoch: RuntimePrepareDeviceEpoch"));
    assert!(readback.contains("RuntimePrepareDeviceEpochMismatch"));
    let epoch_guard = readback
        .find("RuntimePrepareDeviceEpochMismatch")
        .expect("readback epoch guard");
    let diagnostic_admission = readback
        .find("backend.enqueue_product_diagnostic_buffer(")
        .expect("diagnostic admission");
    assert!(epoch_guard < diagnostic_admission);
    assert!(source.contains("backend.enqueue_product_diagnostic_buffer("));
    assert!(!source.contains("GpuReadbackQueue::FRAME_SLOTS"));
    assert!(!source.contains("request_readback_external"));
    assert!(!source.contains("map(<[u8]>::to_vec)"));
    assert!(!readback.contains("map(<[u8]>::to_vec)"));
}

#[test]
fn collector_context_exposes_viewport_size_extract_and_prepared_sidebands() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-runtime-prepare-context-test-encoder"),
    });
    let streamer = test_resource_streamer(&device, &queue);
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(44),
        empty_scene_snapshot(),
    );
    extract.geometry.meshes.push(RenderMeshSnapshot {
        node_id: 9,
        stable_instance_key: 90,
        transform_revision: 1,
        transform: Default::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "tests/runtime-prepare/model",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "tests/runtime-prepare/material",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: Default::default(),
        common: RendererCommon::default(),
    });
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
        test_device_epoch(),
        &mut encoder,
        &streamer,
        &frame,
        &mut external_buffer_bindings,
    );

    assert_eq!(context.viewport_size(), UVec2::new(1280, 720));
    assert_eq!(context.frame_extract().world.raw(), 44);
    assert_eq!(context.scene_snapshot().scene.meshes.len(), 0);
    assert_eq!(context.scene_meshes().len(), 1);
    assert_eq!(context.scene_meshes()[0].node_id, 9);
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
            test_device_epoch(),
            &mut encoder,
            &streamer,
            &frame,
            &mut external_buffer_bindings,
        );
        context.register_external_buffer_binding_with_backing_and_physical_desc(
            "particles.gpu.counters",
            "particles.gpu.counters:test-runtime-prepare",
            &buffer,
            BufferDesc::new("particles.gpu.counters", 32, BufferUsage::STORAGE),
        );
        context.register_static_external_buffer_binding_with_backing(
            "particles.gpu.alive-indices",
            "particles.gpu.alive-indices:test-runtime-prepare",
            &buffer,
        );
        context.register_static_external_buffer_binding_with_backing_and_physical_desc(
            "particles.gpu.particles-a",
            "particles.gpu.particles-a:test-runtime-prepare",
            &buffer,
            BufferDesc::new("particles.gpu.particles-a", 32, BufferUsage::STORAGE),
        );
    }

    assert_eq!(external_buffer_bindings.len(), 3);
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
        external_buffer_bindings[0].physical_desc(),
        Some(desc) if desc.size_bytes == 32 && desc.usage == BufferUsage::STORAGE
    ));
    assert!(matches!(
        &external_buffer_bindings[1].logical_name,
        Cow::Borrowed("particles.gpu.alive-indices")
    ));
    assert!(matches!(
        &external_buffer_bindings[1].backing_name,
        Cow::Borrowed("particles.gpu.alive-indices:test-runtime-prepare")
    ));
    assert!(external_buffer_bindings[1].physical_desc().is_none());
    assert!(matches!(
        &external_buffer_bindings[2].logical_name,
        Cow::Borrowed("particles.gpu.particles-a")
    ));
    assert!(matches!(
        &external_buffer_bindings[2].backing_name,
        Cow::Borrowed("particles.gpu.particles-a:test-runtime-prepare")
    ));
    assert!(matches!(
        external_buffer_bindings[2].physical_desc(),
        Some(desc) if desc.size_bytes == 32 && desc.usage == BufferUsage::STORAGE
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
        test_device_epoch(),
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
        test_device_epoch(),
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
    assert!(
        context
            .request_gpu_readback("test.runtime-prepare", &buffer, 0..4)
            .is_err()
    );
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
                test_device_epoch(),
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
                test_device_epoch(),
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
        test_device_epoch(),
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
