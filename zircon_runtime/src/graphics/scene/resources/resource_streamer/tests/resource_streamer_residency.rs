use std::collections::BTreeMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::{
    RenderExtractContext, RenderFrameExtract, RenderMeshTopology, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest,
};
use crate::core::resource::{
    MeshMarker, ResourceHandle, ResourceId, ResourceKind, ResourceManager, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::render_scene::{
    RenderSceneComponentProjectionCommit, RenderSceneComponentProjector,
};
use crate::scene::components::{ActiveSelf, MeshRenderer, Name, RenderLayerMask};
use crate::scene::World;

use super::super::geometry_replay::RenderSceneGeometryReplayState;
use super::super::ResourceStreamer;

fn replay_test_resource_streamer(
    asset_manager: Arc<ProjectAssetManager>,
    backend: &RenderBackend,
) -> ResourceStreamer {
    let texture_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-render-scene-replay-test-texture-layout"),
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
        asset_manager,
        &backend.device,
        &backend.queue,
        &texture_layout,
    )
}

fn replay_test_mesh(uri: AssetUri, extent: f32) -> MeshAsset {
    MeshAsset::new(
        uri,
        RenderMeshTopology::TriangleList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [extent, 0.0, 0.0],
                    [0.0, extent, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .expect("replay test mesh must be valid")
}

fn replay_test_frame(mesh: ResourceHandle<MeshMarker>) -> RenderFrameExtract {
    let mut world = World::empty();
    let mut renderer = MeshRenderer::default();
    renderer.mesh = Some(mesh);
    world
        .spawn((
            Name("replay-test-mesh".to_string()),
            renderer,
            ActiveSelf::default(),
            RenderLayerMask::default(),
        ))
        .expect("replay test mesh entity must spawn");
    world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(9_104),
        SceneViewportExtractRequest::default(),
    ))
}

#[test]
fn render_scene_admission_reconnects_geometry_replay_and_processes_mesh_update() {
    let backend = RenderBackend::new_offscreen().expect("offscreen render backend");
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mesh_uri = AssetUri::parse("res://meshes/replay-reconnect.zmesh").expect("replay mesh URI");
    let mesh_id = ResourceId::from_locator(&mesh_uri);
    asset_manager
        .resource_manager()
        .register_ready(
            ResourceRecord::new(mesh_id, ResourceKind::Mesh, mesh_uri.clone())
                .with_source_hash("replay-v1"),
            replay_test_mesh(mesh_uri.clone(), 1.0),
        )
        .expect("initial replay mesh publication");

    let mut streamer = replay_test_resource_streamer(Arc::clone(&asset_manager), &backend);
    let frame = replay_test_frame(ResourceHandle::<MeshMarker>::new(mesh_id));
    let mut projector = RenderSceneComponentProjector::new(frame.world);
    let stale_manager = ResourceManager::new();
    streamer.geometry_replays.insert(
        frame.world.raw(),
        RenderSceneGeometryReplayState::new(Some(stale_manager.subscribe())),
    );
    drop(stale_manager);

    streamer
        .admit_render_scene_frame(&mut projector, &backend, &frame, 1)
        .expect("disconnecting replay cursor must resync the frame");
    assert!(streamer
        .geometry_replays
        .get(&frame.world.raw())
        .expect("replay state after disconnect")
        .receiver_missing());

    // Publish while the dead cursor is awaiting its next-frame replacement. The event is
    // intentionally not observable by the replacement receiver; bounded resync must still
    // replay the scene against the latest prepared geometry.
    asset_manager
        .resource_manager()
        .register_ready(
            ResourceRecord::new(mesh_id, ResourceKind::Mesh, mesh_uri.clone())
                .with_source_hash("replay-v2-before-reconnect"),
            replay_test_mesh(mesh_uri.clone(), 2.0),
        )
        .expect("pre-reconnect mesh update publication");

    let reconnect_commit = streamer
        .admit_render_scene_frame(&mut projector, &backend, &frame, 2)
        .expect("next frame must install the replacement receiver");
    let Some(RenderSceneComponentProjectionCommit::Applied { journal, .. }) = reconnect_commit
    else {
        panic!("pre-reconnect mesh update must be recovered by bounded resync")
    };
    assert_eq!(journal.updates().len(), 1);
    let replay = streamer
        .geometry_replays
        .get_mut(&frame.world.raw())
        .expect("replay state after reconnect");
    assert!(!replay.receiver_missing());
    assert!(!replay.drain(0, |_| true).resync);

    asset_manager
        .resource_manager()
        .register_ready(
            ResourceRecord::new(mesh_id, ResourceKind::Mesh, mesh_uri.clone())
                .with_source_hash("replay-v3-after-reconnect"),
            replay_test_mesh(mesh_uri, 3.0),
        )
        .expect("reconnected receiver must observe the mesh update");

    let commit = streamer
        .admit_render_scene_frame(&mut projector, &backend, &frame, 3)
        .expect("mesh update after reconnect must replay");
    let Some(RenderSceneComponentProjectionCommit::Applied { journal, .. }) = commit else {
        panic!("replayed mesh update must publish a geometry journal")
    };
    assert_eq!(journal.updates().len(), 1);

    let replay = streamer
        .geometry_replays
        .get_mut(&frame.world.raw())
        .expect("replay state after mesh update");
    let pending = replay.drain(0, |_| true);
    assert!(!pending.resync);
    assert!(pending.resources.is_empty());
}

#[test]
fn production_residency_bridge_requires_authoritative_delta_inputs() {
    let source = include_str!("../resource_streamer_residency.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("residency bridge must retain a test boundary");

    assert!(production.contains("deltas: &[RenderSceneResourceReferenceDelta]"));
    assert!(production.contains("management: &ResourceManagementGeneration"));
    assert!(production.contains("readiness: &ResourceReadinessGeneration"));
    assert!(production.contains("device: RenderAssetDeviceEpoch"));
    assert!(production.contains("demand_generation: RenderAssetDemandGeneration"));
    assert!(production.contains(".apply_scene_reference_deltas("));
    assert!(!production.contains("ViewportRenderFrame"));
    assert!(!production.contains("frame.meshes()"));
}

#[test]
fn scene_frame_admission_carries_resource_revisions_into_geometry_selection() {
    let source = include_str!("../resource_streamer_residency.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("residency bridge must retain a test boundary");

    assert!(production.contains("prepare_geometry_replay_with_revisions"));
    assert!(production.contains("replay.resources"));
    assert!(!production.contains(".map(|(resource, _)| *resource)"));
}

#[test]
fn scene_frame_admission_bounds_supplemental_geometry_event_retention() {
    let source = include_str!("../resource_streamer_residency.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("residency bridge must retain a test boundary");

    let classify = production
        .find("let artifact_may_reference_supplemental_geometry")
        .expect("model-backed artifacts must classify supplemental dependency risk");
    let drain = production
        .find(".drain(MAX_GEOMETRY_REPLAY_PRIMITIVES")
        .expect("resource cursor must retain its per-frame bound");
    let prepare = production
        .find("self.ensure_render_scene_projection_geometry(&backend.device, artifact)?")
        .expect("changed artifact geometry must still be prepared before projection");
    assert!(classify < drain);
    assert!(drain < prepare);
    assert!(production.contains("artifact_may_reference_supplemental_geometry"));
    assert!(
        production.contains("matches!(resource.kind(), ResourceKind::Mesh | ResourceKind::Model)")
    );
    assert!(production.contains("MAX_GEOMETRY_REPLAY_PRIMITIVES: usize = 256"));
}

#[test]
fn completion_bridge_consumes_one_existing_poll_receipt_without_polling() {
    let source = include_str!("../resource_streamer_residency.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("residency bridge must retain a test boundary");

    assert!(production.contains("poll_receipt: SubmissionPollReceipt"));
    assert!(production.contains(".maintain_gpu_after_rhi_poll("));
    assert!(production.contains("last_render_asset_gpu_maintenance = report"));
    assert!(!production.contains("poll_submission_completions"));
    assert!(!production.contains("device.poll("));
}

#[test]
fn scene_frame_admission_retains_requests_and_cancels_unstarted_work() {
    let source = include_str!("../resource_streamer_residency.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("residency bridge must retain a test boundary");

    assert!(production.contains("project_frame_with_resource_geometry_staging"));
    assert!(production.contains("projection_snapshot()"));
    assert!(production.contains("render_asset_residency_work_queue.retain_mutation"));
    assert!(production.contains("has_resource_replay"));
    assert!(production.contains("let replay_limit = MAX_GEOMETRY_REPLAY_PRIMITIVES"));
    assert!(production.contains("!selection.truncated()"));
    assert!(!production.contains("else { usize::MAX }"));
    assert!(!production.contains("pending_render_asset_residency_retirements"));
    assert!(!production.contains("frame.meshes()"));

    let replay = production
        .find("projector.is_exact_frame_replay(frame)")
        .expect("stable multi-view replay must be identified first");
    let geometry = production
        .find("self.ensure_render_scene_projection_geometry")
        .expect("changed artifacts must resolve all-LOD geometry");
    let projection = production
        .find("projection_snapshot()")
        .expect("changed artifacts must capture resource generations");
    assert!(replay < geometry);
    assert!(geometry < projection);
}

#[test]
fn world_release_is_staged_before_registry_removal() {
    let source = include_str!("../resource_streamer_residency.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("residency bridge must retain a test boundary");

    let release = production
        .find("release_render_scene_world")
        .expect("world release owner");
    let staging = production[release..]
        .find(".release_world_with_staging(")
        .expect("registry transaction");
    let residency = production[release..]
        .find(".apply_scene_reference_deltas(")
        .expect("residency release admission");
    assert!(staging < residency);
    assert!(!production.contains("viewports.remove"));
}
