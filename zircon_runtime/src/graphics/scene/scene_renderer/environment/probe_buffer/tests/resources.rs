use std::sync::Arc;

use crate::asset::{
    texture_asset_from_ibl_bake_artifact_pmrem, AssetUri, ProjectAssetManager, TextureAsset,
};
use crate::core::framework::render::{
    build_source_cubemap_from_equirect, IblBakeArtifactBlob, IblBakeArtifactContents,
    IblBakeArtifactDescriptor, IblBakeArtifactPayload, PlanarReflectionProbeData, PlanarUpdateMode,
    ProbeInfluenceShape, ProceduralSkyParams, ReflectionProbeData, RenderCameraTarget,
    RenderLayerSet,
};
use crate::core::math::{Mat4, Quat, UVec2, Vec3};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;
use crate::scene::world::World;

use super::super::capacity::ReflectionProbeResourceCapacity;
use super::super::reflection_probe_bind_group_layout_entries;
use super::super::resources::{
    SceneReflectionProbeResources, MAX_REFLECTION_PROBES, REFLECTION_PROBE_FACE_COUNT,
    REFLECTION_PROBE_FACE_SIZE, REFLECTION_PROBE_MIP_COUNT,
};
use super::super::upload::ReflectionProbeAssetRejectionReason;

#[test]
fn render_probe_gpu_capacity_matches_plan_v1_limit() {
    assert_eq!(MAX_REFLECTION_PROBES, 64);
    assert_eq!(ReflectionProbeResourceCapacity::FULL.probe_count, 64);
    assert_eq!(ReflectionProbeResourceCapacity::FULL.cubemap_slot_count, 65);
    assert_eq!(
        ReflectionProbeResourceCapacity::ENVIRONMENT_PREVIEW_PLACEHOLDER.cubemap_slot_count,
        2
    );
}

#[test]
fn render_probe_prepare_reads_registry_before_candidate_upload_loop() {
    let source = include_str!("../resources.rs");
    let prepare_start = source
        .find("fn prepare(")
        .expect("probe prepare implementation");
    let prepare_end = source[prepare_start..]
        .find("fn append_buffer_uploads")
        .map(|offset| prepare_start + offset)
        .expect("probe prepare boundary");
    let prepare = &source[prepare_start..prepare_end];
    let registry_read = prepare
        .find("resource_manager.registry()")
        .expect("probe registry read");
    let capacity_partition = prepare
        .find("select_nth_unstable_by(")
        .expect("over-capacity probes must partition before registry resolution");
    let asset_manager = prepare
        .find("let asset_manager = match streamer.asset_manager()")
        .expect("asset manager resolution");
    let candidate_loop = prepare
        .find("in candidates {")
        .expect("probe candidate upload loop");

    assert!(
        capacity_partition < asset_manager
            && asset_manager < registry_read
            && registry_read < candidate_loop,
        "probe prepare must partition before resolving asset-manager or registry state, then resolve only selected candidate revisions before loading assets"
    );
}

#[test]
fn capture_probe_copy_covers_all_pmrem_mips_without_committing_slot_state() {
    let source = include_str!("../resources.rs");
    let copy_start = source
        .find("fn copy_environment_capture_probe(")
        .expect("capture copy owner");
    let copy_end = source[copy_start..]
        .find("fn commit_environment_capture_target(")
        .map(|offset| copy_start + offset)
        .expect("capture copy must precede its explicit commit owner");
    let copy = &source[copy_start..copy_end];

    assert!(copy.contains("for mip_level in 0..REFLECTION_PROBE_MIP_COUNT"));
    assert!(copy.contains("encoder.copy_texture_to_texture("));
    assert!(!copy.contains("commit_pending_uploads"));
}

#[test]
fn render_probe_prepare_clears_header_when_asset_manager_resolution_fails() {
    let source = include_str!("../resources.rs");
    let prepare_start = source
        .find("fn prepare(")
        .expect("probe prepare implementation");
    let prepare_end = source[prepare_start..]
        .find("fn append_buffer_uploads")
        .map(|offset| prepare_start + offset)
        .expect("probe prepare boundary");
    let prepare = &source[prepare_start..prepare_end];
    let asset_manager = prepare
        .find("let asset_manager = match streamer.asset_manager()")
        .expect("probe prepare must resolve the asset manager before baked-probe upload");
    let unavailable = prepare[asset_manager..]
        .find("Err(_) => {")
        .map(|offset| asset_manager + offset)
        .expect("asset-manager failure must have an explicit recovery branch");
    let clear_header = prepare[unavailable..]
        .find("self.append_buffer_uploads(")
        .map(|offset| unavailable + offset)
        .expect("asset-manager failure must clear the prior probe header");
    let clear_header_call = prepare[clear_header..]
        .find("&[],\n                    camera_layers.to_scene_schema_v1_mask_lossy(),")
        .map(|offset| clear_header + offset)
        .expect("asset-manager failure must clear probes while preserving the camera layer mask");
    let return_report = prepare[clear_header..]
        .find("return report;")
        .map(|offset| clear_header + offset)
        .expect("asset-manager recovery must return after clearing stale probe state");

    assert!(
        asset_manager < unavailable
            && unavailable < clear_header
            && clear_header < clear_header_call
            && clear_header_call < return_report,
        "unavailable asset-manager recovery must clear probe visibility before returning"
    );
}

#[test]
fn render_probe_candidate_distance_rotates_only_box_influences() {
    let source = include_str!("../selection.rs");
    let distance = source
        .split("fn probe_distance_to_influence")
        .nth(1)
        .and_then(|text| text.split("fn record_probe_asset_rejection").next())
        .expect("probe candidate-distance helper");

    let position_delta = distance
        .find("let position_delta = world_position - probe.position();")
        .expect("candidate distance must retain the world-space center delta");
    let box_branch = distance
        .find("ProbeInfluenceShape::Box { half_extents, .. } => {")
        .expect("candidate distance must retain box influence distance");
    let box_rotation = distance
        .find("let local = probe.rotation().conjugate() * position_delta;")
        .expect("box candidate distance must retain local-space rotation");
    let sphere_branch = distance
        .find("ProbeInfluenceShape::Sphere { radius, .. } =>")
        .expect("candidate distance must retain sphere influence distance");
    let sphere_distance = distance
        .find("(position_delta.length() - radius).max(0.0)")
        .expect("sphere candidate distance must use its rotation-invariant center distance");

    assert!(
        position_delta < box_branch
            && box_branch < box_rotation
            && box_rotation < sphere_branch
            && sphere_branch < sphere_distance,
        "only box candidate distances may rotate the world-space center delta"
    );
}

#[test]
fn render_probe_prepare_evaluates_candidate_distance_once_before_sorting() {
    let source = include_str!("../resources.rs");
    let prepare_start = source
        .find("fn prepare(")
        .expect("probe prepare implementation");
    let prepare_end = source[prepare_start..]
        .find("fn append_buffer_uploads")
        .map(|offset| prepare_start + offset)
        .expect("probe prepare boundary");
    let prepare = &source[prepare_start..prepare_end];
    let sort = prepare
        .find("candidates.sort_by")
        .expect("candidate distance sort");
    let candidate_loop = prepare
        .find("in candidates {")
        .expect("probe candidate upload loop");
    let distance_call = "probe_distance_to_influence(probe, camera_position)";

    assert_eq!(
        prepare[..sort].matches(distance_call).count(),
        1,
        "each eligible probe must calculate its distance once before sorting"
    );
    assert!(
        prepare[..sort].contains("ReflectionProbeCandidate {"),
        "distance caching must stay inside the eligible-candidate closure"
    );
    let sort_body = &prepare[sort..candidate_loop];
    assert!(
        sort_body.contains("reflection_probe_candidate_order"),
        "candidate sorting must use the cached-candidate comparator"
    );
    assert!(
        !sort_body.contains(distance_call),
        "candidate sorting must not recalculate geometry for each comparison"
    );
    assert!(
        source.contains("right.probe.priority().cmp(&left.probe.priority())")
            && source.contains("left.probe.probe_id().cmp(&right.probe.probe_id())")
            && source.contains("left.cubemap.cmp(&right.cubemap)")
            && source.contains("left.extraction_order.cmp(&right.extraction_order)"),
        "candidate sorting must retain a total deterministic order after duplicate probe IDs"
    );
}

#[test]
fn render_probe_prepare_partitions_over_capacity_candidates_before_final_sort() {
    let source = include_str!("../resources.rs");
    let prepare_start = source
        .find("fn prepare(")
        .expect("probe prepare implementation");
    let prepare_end = source[prepare_start..]
        .find("fn append_buffer_uploads")
        .map(|offset| prepare_start + offset)
        .expect("probe prepare boundary");
    let prepare = &source[prepare_start..prepare_end];

    let capacity_guard = prepare
        .find("if candidates.len() > MAX_REFLECTION_PROBES {")
        .expect("candidate selection must branch only when capacity is exceeded");
    let partition = prepare
        .find("select_nth_unstable_by(")
        .expect("over-capacity candidate selection must partition instead of full-sorting");
    let overflow = prepare
        .find("candidates.split_off(MAX_REFLECTION_PROBES)")
        .expect("partitioned candidates must retain their overflow for invalid-asset replacement");
    let final_sort = prepare
        .rfind("candidates.sort_by(reflection_probe_candidate_order);")
        .expect("the selected candidates must retain deterministic upload ordering");

    assert!(
        capacity_guard < partition && partition < overflow && overflow < final_sort,
        "candidate selection must partition then retain overflow before the final deterministic sort"
    );
    let partition_body = &prepare[partition..overflow];
    assert!(
        partition_body.contains("MAX_REFLECTION_PROBES")
            && partition_body.contains("reflection_probe_candidate_order"),
        "the partition must use the configured capacity and the canonical candidate comparator"
    );
    let replacement = prepare
        .find("overflow_candidates.sort_by(reflection_probe_candidate_order);")
        .expect("invalid selected candidates must sort overflow once before ordered replacement");
    assert!(
        final_sort < replacement,
        "overflow replacement must preserve the sorted first-capacity fast path"
    );
}

#[test]
fn render_probe_resources_upload_valid_pmrem_once_and_disable_to_sky_fallback() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/test-probe-pmrem.zcube").expect("valid cubemap URI");
    let cubemap = ResourceId::from_locator(&cubemap_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(cubemap, ResourceKind::Texture, cubemap_uri.clone()),
            valid_probe_pmrem(cubemap_uri),
        )
        .expect("probe PMREM insert");
    let streamer = ResourceStreamer::new_for_test(
        Arc::clone(&asset_manager),
        &device,
        &queue,
        &texture_layout,
    );
    let frame = probe_frame(cubemap);
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut resources = SceneReflectionProbeResources::new(&device);

    let first = prepare_resources(&mut resources, &backend, &streamer, &frame, true);
    assert_eq!(first.extracted_probe_count, 1);
    assert_eq!(first.camera_layer_candidate_count, 1);
    assert_eq!(first.attempted_candidate_count, 1);
    assert_eq!(first.capacity_dropped_candidate_count, 0);
    assert_eq!(first.active_probe_count, 1);
    assert_eq!(first.scheduled_cubemap_upload_count, 1);
    assert_eq!(
        first.scheduled_texture_write_count,
        REFLECTION_PROBE_MIP_COUNT as usize
    );
    assert_eq!(first.asset_load_call_count, 1);
    let expected_upload_bytes = (0..REFLECTION_PROBE_MIP_COUNT)
        .map(|mip| {
            let edge = (REFLECTION_PROBE_FACE_SIZE >> mip).max(1) as u64;
            edge * edge * u64::from(REFLECTION_PROBE_FACE_COUNT) * 8
        })
        .sum::<u64>();
    assert_eq!(first.scheduled_cubemap_upload_bytes, expected_upload_bytes);
    assert_eq!(first.rejected_cubemap_count, 0);
    assert_eq!(first.first_rejection, None);

    let second = prepare_resources(&mut resources, &backend, &streamer, &frame, true);
    assert_eq!(second.active_probe_count, 1);
    assert_eq!(second.scheduled_cubemap_upload_count, 0);
    assert_eq!(second.scheduled_cubemap_upload_bytes, 0);
    assert_eq!(second.scheduled_texture_write_count, 0);
    assert_eq!(second.asset_load_call_count, 0);
    assert_eq!(second.asset_load_cpu_time_us, 0);
    assert_eq!(second.rejected_cubemap_count, 0);

    let disabled = prepare_resources(&mut resources, &backend, &streamer, &frame, false);
    assert_eq!(disabled.extracted_probe_count, 1);
    assert_eq!(disabled.camera_layer_candidate_count, 0);
    assert_eq!(disabled.attempted_candidate_count, 0);
    assert_eq!(disabled.active_probe_count, 0);
    assert_eq!(disabled.scheduled_cubemap_upload_count, 0);
    assert_eq!(disabled.scheduled_texture_write_count, 0);
    assert_eq!(disabled.asset_load_call_count, 0);
    assert_eq!(disabled.asset_load_cpu_time_us, 0);
    assert_eq!(disabled.rejected_cubemap_count, 0);

    let validation_error = pollster::block_on(error_scope.pop());
    assert!(
        validation_error.is_none(),
        "probe texture upload and bindings should pass WGPU validation: {validation_error:?}"
    );
}

#[test]
fn render_probe_resources_retry_an_uncommitted_cubemap_upload() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/uncommitted-probe-pmrem.zcube").expect("valid URI");
    let cubemap = ResourceId::from_locator(&cubemap_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(cubemap, ResourceKind::Texture, cubemap_uri.clone()),
            valid_probe_pmrem(cubemap_uri),
        )
        .expect("probe PMREM insert");
    let streamer = ResourceStreamer::new_for_test(asset_manager, device, queue, &texture_layout);
    let frame = probe_frame(cubemap);
    let mut resources = SceneReflectionProbeResources::new(device);

    let mut first_buffer_uploads = zr_rhi_wgpu::WgpuBufferUploadBatch::new();
    let mut first_texture_uploads = zr_rhi_wgpu::WgpuTextureUploadBatch::new();
    let first = resources.prepare(
        device,
        &streamer,
        &frame,
        true,
        &mut first_buffer_uploads,
        &mut first_texture_uploads,
    );
    assert_eq!(first.scheduled_cubemap_upload_count, 1);
    assert_eq!(
        first.scheduled_texture_write_count,
        REFLECTION_PROBE_MIP_COUNT as usize
    );

    resources.discard_pending_uploads();
    drop((first_buffer_uploads, first_texture_uploads));

    let mut retry_buffer_uploads = zr_rhi_wgpu::WgpuBufferUploadBatch::new();
    let mut retry_texture_uploads = zr_rhi_wgpu::WgpuTextureUploadBatch::new();
    let retry = resources.prepare(
        device,
        &streamer,
        &frame,
        true,
        &mut retry_buffer_uploads,
        &mut retry_texture_uploads,
    );

    assert_eq!(retry.active_probe_count, 1);
    assert_eq!(retry.scheduled_cubemap_upload_count, 1);
    assert_eq!(retry.asset_load_call_count, 1);
    assert_eq!(
        retry.scheduled_texture_write_count,
        REFLECTION_PROBE_MIP_COUNT as usize
    );
    resources.discard_pending_uploads();
}

#[test]
fn environment_preview_defers_local_provider_resources_until_a_probe_is_enabled() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/deferred-preview-probe.zcube").expect("valid URI");
    let cubemap = ResourceId::from_locator(&cubemap_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(cubemap, ResourceKind::Texture, cubemap_uri.clone()),
            valid_probe_pmrem(cubemap_uri),
        )
        .expect("probe PMREM insert");
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let frame = probe_frame(cubemap);
    let mut resources = SceneReflectionProbeResources::new_environment_only_preview(&device);

    assert!(resources.is_environment_only_placeholder_for_tests());
    assert!(!resources.requires_generic_environment_pbr());
    let disabled = prepare_resources(&mut resources, &backend, &streamer, &frame, false);
    assert_eq!(disabled.active_probe_count, 0);
    assert!(resources.is_environment_only_placeholder_for_tests());

    let enabled = prepare_resources(&mut resources, &backend, &streamer, &frame, true);
    assert_eq!(enabled.active_probe_count, 1);
    assert_eq!(enabled.scheduled_cubemap_upload_count, 1);
    assert_eq!(enabled.asset_load_call_count, 1);
    assert!(!resources.is_environment_only_placeholder_for_tests());
    assert!(resources.requires_generic_environment_pbr());

    let hidden = prepare_resources(&mut resources, &backend, &streamer, &frame, false);
    assert_eq!(hidden.active_probe_count, 0);
    assert!(
        resources.requires_generic_environment_pbr(),
        "a provider upgrade remains sticky so Base variants cannot thrash between environment ABIs"
    );
}

#[test]
fn environment_preview_does_not_upgrade_for_a_rejected_baked_probe() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/rejected-preview-probe.zcube").expect("valid URI");
    let cubemap = ResourceId::from_locator(&cubemap_uri);
    let source_asset = crate::asset::texture_asset_from_source_cubemap_zcube(
        cubemap_uri.clone(),
        &build_source_cubemap_from_equirect(REFLECTION_PROBE_FACE_SIZE, |_, _| {
            [0.2, 0.4, 0.8, 1.0]
        }),
    );
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(cubemap, ResourceKind::Texture, cubemap_uri),
            source_asset,
        )
        .expect("source cubemap insert");
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let frame = probe_frame(cubemap);
    let mut resources = SceneReflectionProbeResources::new_environment_only_preview(&device);

    let report = prepare_resources(&mut resources, &backend, &streamer, &frame, true);

    assert_eq!(report.active_probe_count, 0);
    assert_eq!(report.scheduled_cubemap_upload_count, 0);
    assert_eq!(report.scheduled_texture_write_count, 0);
    assert_eq!(report.asset_load_call_count, 1);
    assert_eq!(report.rejected_cubemap_count, 1);
    assert!(
        resources.is_environment_only_placeholder_for_tests(),
        "a rejected baked probe must not allocate full local-provider resources"
    );
    assert!(
        !resources.requires_generic_environment_pbr(),
        "a rejected baked probe must not select the generic PBR variant"
    );
}

#[test]
fn full_scene_reflection_resources_do_not_report_an_environment_preview_upgrade() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let resources = SceneReflectionProbeResources::new(&device);

    assert!(
        !resources.requires_generic_environment_pbr(),
        "full-scene resource capacity is not an environment-preview provider upgrade"
    );
}

#[test]
fn environment_capture_expands_placeholder_before_pmrem_array_copy() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let mut resources = SceneReflectionProbeResources::new_environment_only_preview(device);

    assert!(resources.is_environment_only_placeholder_for_tests());
    resources.ensure_environment_capture_provider(device);
    assert!(!resources.is_environment_only_placeholder_for_tests());
    assert!(resources.requires_generic_environment_pbr());
}

#[test]
fn environment_preview_placeholder_satisfies_the_local_provider_binding_abi() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let resources = SceneReflectionProbeResources::new_environment_only_preview(&device);
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-environment-preview-probe-placeholder-layout"),
        entries: &reflection_probe_bind_group_layout_entries(),
    });
    let bindings = resources.bindings();
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-environment-preview-probe-placeholder-bind-group"),
        layout: &layout,
        entries: &bindings.bind_group_entries(),
    });

    let validation_error = pollster::block_on(error_scope.pop());
    assert!(
        validation_error.is_none(),
        "environment preview placeholder must satisfy the local-provider binding ABI: {validation_error:?}"
    );
    let diagnostic_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    resources
        .gpu_upload_diagnostics(&device, &queue)
        .expect("environment preview placeholder diagnostics must stay in bounds");
    let diagnostic_error = pollster::block_on(diagnostic_scope.pop());
    assert!(
        diagnostic_error.is_none(),
        "environment preview diagnostics must not issue invalid readbacks: {diagnostic_error:?}"
    );
}

#[test]
fn environment_preview_upgrades_for_a_planar_capture_camera_and_rebinds() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let capture_target = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://render-targets/environment-preview-planar-capture.ztexture",
    ));
    let mut frame = probe_frame(ResourceId::from_stable_label("builtin://unused-probe"));
    let extract = Arc::make_mut(&mut frame.extract);
    extract.environment.probes.clear();
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("probe fixture has a selected camera")
        .target = RenderCameraTarget::Texture(capture_target.clone());
    extract.lighting.advanced_lighting.planar_probes = vec![PlanarReflectionProbeData {
        probe_id: 9,
        plane_transform: Mat4::IDENTITY,
        local_reference_position: Vec3::ZERO,
        bounds_min: Vec3::splat(-1.0),
        bounds_max: Vec3::splat(1.0),
        resolution: 256,
        update: PlanarUpdateMode::EveryFrame,
        capture_target: Some(capture_target),
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }];
    let mut resources = SceneReflectionProbeResources::new_environment_only_preview(&device);

    assert!(!resources.requires_generic_environment_pbr());
    let report = prepare_resources(&mut resources, &backend, &streamer, &frame, false);
    assert_eq!(report.active_probe_count, 0);
    assert!(!resources.is_environment_only_placeholder_for_tests());
    assert!(resources.requires_generic_environment_pbr());

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-environment-preview-planar-upgrade-layout"),
        entries: &reflection_probe_bind_group_layout_entries(),
    });
    let bindings = resources.bindings();
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-environment-preview-planar-upgrade-bind-group"),
        layout: &layout,
        entries: &bindings.bind_group_entries(),
    });
    let validation_error = pollster::block_on(error_scope.pop());
    assert!(
        validation_error.is_none(),
        "upgraded planar provider bindings must satisfy the local-provider ABI: {validation_error:?}"
    );
}

#[test]
fn environment_preview_selects_the_lowest_id_valid_planar_provider() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let capture_target = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://render-targets/environment-preview-planar-selection.ztexture",
    ));
    let mut frame = probe_frame(ResourceId::from_stable_label("builtin://unused-probe"));
    let extract = Arc::make_mut(&mut frame.extract);
    extract.environment.probes.clear();
    extract.lighting.advanced_lighting.planar_probes = vec![
        PlanarReflectionProbeData {
            probe_id: 1,
            plane_transform: Mat4::ZERO,
            local_reference_position: Vec3::ZERO,
            bounds_min: Vec3::splat(-1.0),
            bounds_max: Vec3::splat(1.0),
            resolution: 256,
            update: PlanarUpdateMode::EveryFrame,
            capture_target: Some(capture_target.clone()),
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        },
        PlanarReflectionProbeData {
            probe_id: 2,
            plane_transform: Mat4::IDENTITY,
            local_reference_position: Vec3::ZERO,
            bounds_min: Vec3::splat(-1.0),
            bounds_max: Vec3::splat(1.0),
            resolution: 128,
            update: PlanarUpdateMode::EveryFrame,
            capture_target: Some(capture_target.clone()),
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        },
        PlanarReflectionProbeData {
            probe_id: 3,
            plane_transform: Mat4::IDENTITY,
            local_reference_position: Vec3::ZERO,
            bounds_min: Vec3::splat(-1.0),
            bounds_max: Vec3::splat(1.0),
            resolution: 512,
            update: PlanarUpdateMode::EveryFrame,
            capture_target: Some(capture_target),
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        },
    ];
    let mut resources = SceneReflectionProbeResources::new_environment_only_preview(&device);

    let report = prepare_resources(&mut resources, &backend, &streamer, &frame, false);

    assert_eq!(report.active_probe_count, 0);
    assert!(resources.requires_generic_environment_pbr());
    let params = resources
        .gpu_planar_params_for_tests(&device, &queue)
        .expect("planar parameter readback");
    assert_eq!(
        params.sample_params[0], 0.125,
        "the lowest-ID valid planar provider must control the uploaded parameters"
    );
}

#[test]
fn render_probe_resources_report_source_cubemap_rejection() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/source-only.zcube").expect("valid cubemap URI");
    let cubemap = ResourceId::from_locator(&cubemap_uri);
    let source_asset = crate::asset::texture_asset_from_source_cubemap_zcube(
        cubemap_uri.clone(),
        &crate::core::framework::render::build_source_cubemap_from_equirect(
            REFLECTION_PROBE_FACE_SIZE,
            |_, _| [0.2, 0.4, 0.8, 1.0],
        ),
    );
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(cubemap, ResourceKind::Texture, cubemap_uri),
            source_asset,
        )
        .expect("source cubemap insert");
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let mut resources = SceneReflectionProbeResources::new(&device);

    let report = prepare_resources(
        &mut resources,
        &backend,
        &streamer,
        &probe_frame(cubemap),
        true,
    );

    assert_eq!(report.active_probe_count, 0);
    assert_eq!(report.scheduled_cubemap_upload_count, 0);
    assert_eq!(report.rejected_cubemap_count, 1);
    assert_eq!(
        report
            .first_rejection
            .expect("typed rejection should be retained")
            .reason,
        ReflectionProbeAssetRejectionReason::SourceCubemapRequiresPrefiltering
    );
}

#[test]
fn render_probe_over_capacity_replaces_an_invalid_nearest_candidate() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/overflow-valid-probe.zcube").expect("valid URI");
    let valid_cubemap = ResourceId::from_locator(&cubemap_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(valid_cubemap, ResourceKind::Texture, cubemap_uri.clone()),
            valid_probe_pmrem(cubemap_uri),
        )
        .expect("valid PMREM insert");
    let invalid_cubemap = ResourceId::from_stable_label("builtin://missing-overflow-probe");
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let shape = ProbeInfluenceShape::sphere(1.0, 0.0).expect("valid sphere influence");
    let mut extract = World::new().to_render_frame_extract();
    extract.environment.probes = (0..=MAX_REFLECTION_PROBES)
        .map(|index| {
            ReflectionProbeData::try_new(
                index as u64,
                Vec3::new(index as f32 * 4.0, 0.0, 0.0),
                Quat::IDENTITY,
                shape,
                Vec3::splat(1.0),
            )
            .expect("valid overflow probe")
            .with_baked_cubemap(Some(if index == 0 {
                invalid_cubemap
            } else {
                valid_cubemap
            }))
        })
        .collect();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
    let mut resources = SceneReflectionProbeResources::new(&device);

    let report = prepare_resources(&mut resources, &backend, &streamer, &frame, true);

    assert_eq!(report.extracted_probe_count, MAX_REFLECTION_PROBES + 1);
    assert_eq!(
        report.camera_layer_candidate_count,
        MAX_REFLECTION_PROBES + 1
    );
    assert_eq!(report.attempted_candidate_count, MAX_REFLECTION_PROBES + 1);
    assert_eq!(report.capacity_dropped_candidate_count, 0);
    assert_eq!(report.active_probe_count, MAX_REFLECTION_PROBES);
    assert_eq!(report.scheduled_cubemap_upload_count, 1);
    assert_eq!(
        report.scheduled_texture_write_count,
        REFLECTION_PROBE_MIP_COUNT as usize
    );
    assert_eq!(report.asset_load_call_count, 1);
    assert_eq!(report.rejected_cubemap_count, 1);
    assert_eq!(
        report
            .first_rejection
            .expect("nearest invalid candidate rejection")
            .reason,
        ReflectionProbeAssetRejectionReason::MissingResource
    );
}

#[test]
fn render_probe_over_capacity_resolves_only_selected_healthy_candidates() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let cubemap_uri =
        AssetUri::parse("res://environment/selected-probe-only.zcube").expect("valid URI");
    let cubemap = ResourceId::from_locator(&cubemap_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(cubemap, ResourceKind::Texture, cubemap_uri.clone()),
            valid_probe_pmrem(cubemap_uri),
        )
        .expect("valid PMREM insert");
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let shape = ProbeInfluenceShape::sphere(1.0, 0.0).expect("valid sphere influence");
    let mut extract = World::new().to_render_frame_extract();
    extract.environment.probes = (0..=MAX_REFLECTION_PROBES)
        .map(|index| {
            ReflectionProbeData::try_new(
                index as u64,
                Vec3::new(index as f32 * 4.0, 0.0, 0.0),
                Quat::IDENTITY,
                shape,
                Vec3::splat(1.0),
            )
            .expect("valid selected probe")
            .with_baked_cubemap(Some(cubemap))
        })
        .collect();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
    let mut resources = SceneReflectionProbeResources::new(&device);

    let report = prepare_resources(&mut resources, &backend, &streamer, &frame, true);

    assert_eq!(
        report.camera_layer_candidate_count,
        MAX_REFLECTION_PROBES + 1
    );
    assert_eq!(report.attempted_candidate_count, MAX_REFLECTION_PROBES);
    assert_eq!(report.capacity_dropped_candidate_count, 1);
    assert_eq!(report.active_probe_count, MAX_REFLECTION_PROBES);
    assert_eq!(
        report.scheduled_texture_write_count,
        REFLECTION_PROBE_MIP_COUNT as usize
    );
    assert_eq!(report.asset_load_call_count, 1);
    assert_eq!(
        resources.candidate_registry_resolution_count_for_tests(),
        MAX_REFLECTION_PROBES,
        "healthy overflow candidates must not touch the registry or asset path"
    );
}

fn prepare_resources(
    resources: &mut SceneReflectionProbeResources,
    backend: &RenderBackend,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    enabled: bool,
) -> super::super::resources::ReflectionProbeUploadReport {
    let mut frame_buffer_uploads = zr_rhi_wgpu::WgpuBufferUploadBatch::new();
    let mut frame_texture_uploads = zr_rhi_wgpu::WgpuTextureUploadBatch::new();
    let report = resources.prepare(
        &backend.device,
        streamer,
        frame,
        enabled,
        &mut frame_buffer_uploads,
        &mut frame_texture_uploads,
    );
    backend
        .enqueue_copy_resource_upload_batch(zr_rhi_wgpu::WgpuResourceUploadBatch::from_batches(
            frame_buffer_uploads,
            frame_texture_uploads,
        ))
        .expect("probe test frame uploads should be accepted");
    backend
        .submit_graphics_command_buffers(Vec::new())
        .expect("probe test frame uploads should reach the native queue");
    resources.commit_pending_uploads();
    report
}

fn valid_probe_pmrem(uri: AssetUri) -> TextureAsset {
    let source = build_source_cubemap_from_equirect(REFLECTION_PROBE_FACE_SIZE, |u, v| {
        [2.0 + u * 3.0, 0.25 + v, 0.5, 1.0]
    });
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor = IblBakeArtifactDescriptor::current(
        key,
        REFLECTION_PROBE_FACE_SIZE,
        REFLECTION_PROBE_MIP_COUNT,
        IblBakeArtifactContents::PMREM,
    );
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &source, None)
        .expect("valid PMREM payload");
    texture_asset_from_ibl_bake_artifact_pmrem(uri, &IblBakeArtifactBlob::from_payload(payload))
        .expect("current PMREM texture")
}

fn probe_frame(cubemap: ResourceId) -> ViewportRenderFrame {
    let shape =
        ProbeInfluenceShape::box_shape(Vec3::splat(4.0), 1.0).expect("valid probe influence");
    let probe =
        ReflectionProbeData::try_new(7, Vec3::ZERO, Quat::IDENTITY, shape, Vec3::splat(4.0))
            .expect("valid reflection probe")
            .with_box_projection(true)
            .with_baked_cubemap(Some(cubemap));
    let mut extract = World::new().to_render_frame_extract();
    extract.environment.probes = vec![probe];
    ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-reflection-probe-test-texture-layout"),
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
    })
}
