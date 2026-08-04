use std::sync::Arc;

use crate::asset::{
    AssetUri, ProjectAssetManager, TextureAsset, texture_asset_from_ibl_bake_artifact_pmrem,
};
use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, PlanarReflectionProbeData, PlanarUpdateMode, ProbeInfluenceShape,
    ProceduralSkyParams, ReflectionProbeData, RenderCameraTarget, RenderLayerSet,
    build_source_cubemap_from_equirect,
};
use crate::core::math::{Mat4, Quat, UVec2, Vec3};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;
use crate::scene::world::World;

use super::super::reflection_probe_bind_group_layout_entries;
use super::super::resources::{
    MAX_REFLECTION_PROBES, REFLECTION_PROBE_FACE_SIZE, REFLECTION_PROBE_MIP_COUNT,
    SceneReflectionProbeResources,
};
use super::super::upload::ReflectionProbeAssetRejectionReason;

#[test]
fn render_probe_gpu_capacity_matches_plan_v1_limit() {
    assert_eq!(MAX_REFLECTION_PROBES, 64);
}

#[test]
fn render_probe_prepare_reads_registry_before_candidate_upload_loop() {
    let source = include_str!("../resources.rs");
    let prepare_start = source
        .find("fn prepare(")
        .expect("probe prepare implementation");
    let prepare_end = source[prepare_start..]
        .find("fn write_probe_header")
        .map(|offset| prepare_start + offset)
        .expect("probe prepare boundary");
    let prepare = &source[prepare_start..prepare_end];
    let registry_read = prepare
        .find("resource_manager.registry()")
        .expect("probe registry read");
    let candidate_loop = prepare
        .find("in candidates {")
        .expect("probe candidate upload loop");

    assert!(
        registry_read < candidate_loop,
        "probe prepare must read candidate revisions under one short registry lock before loading assets"
    );
}

#[test]
fn render_probe_candidate_distance_rotates_only_box_influences() {
    let source = include_str!("../resources.rs");
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
        .find("fn write_probe_header")
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
        prepare[..sort].contains(".then(|| {"),
        "distance caching must stay inside the eligible-candidate closure"
    );
    let sort_body = &prepare[sort..candidate_loop];
    assert!(
        sort_body.contains("left.3.total_cmp(&right.3)"),
        "candidate sorting must compare the cached distance"
    );
    assert!(
        !sort_body.contains(distance_call),
        "candidate sorting must not recalculate geometry for each comparison"
    );
    assert!(
        sort_body.contains("right.0.priority().cmp(&left.0.priority())")
            && sort_body.contains("left.0.probe_id().cmp(&right.0.probe_id())"),
        "candidate sorting must retain priority and probe-ID tie ordering"
    );
}

#[test]
fn render_probe_prepare_partitions_over_capacity_candidates_before_final_sort() {
    let source = include_str!("../resources.rs");
    let prepare_start = source
        .find("fn prepare(")
        .expect("probe prepare implementation");
    let prepare_end = source[prepare_start..]
        .find("fn write_probe_header")
        .map(|offset| prepare_start + offset)
        .expect("probe prepare boundary");
    let prepare = &source[prepare_start..prepare_end];

    let capacity_guard = prepare
        .find("if candidates.len() > MAX_REFLECTION_PROBES {")
        .expect("candidate selection must branch only when capacity is exceeded");
    let partition = prepare
        .find("candidates.select_nth_unstable_by(")
        .expect("over-capacity candidate selection must partition instead of full-sorting");
    let truncate = prepare
        .find("candidates.truncate(MAX_REFLECTION_PROBES);")
        .expect("partitioned candidates must retain only the configured capacity");
    let final_sort = prepare
        .rfind("candidates.sort_by(candidate_order);")
        .expect("the selected candidates must retain deterministic upload ordering");

    assert!(
        capacity_guard < partition && partition < truncate && truncate < final_sort,
        "candidate selection must partition then truncate before the final deterministic sort"
    );
    let partition_body = &prepare[partition..truncate];
    assert!(
        partition_body.contains("MAX_REFLECTION_PROBES")
            && partition_body.contains("candidate_order"),
        "the partition must use the configured capacity and the canonical candidate comparator"
    );
}

#[test]
fn render_probe_resources_upload_valid_pmrem_once_and_disable_to_sky_fallback() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
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

    let first = resources.prepare(&device, &queue, &streamer, &frame, true);
    assert_eq!(first.extracted_probe_count, 1);
    assert_eq!(first.active_probe_count, 1);
    assert_eq!(first.uploaded_cubemap_count, 1);
    assert_eq!(first.rejected_cubemap_count, 0);
    assert_eq!(first.first_rejection, None);

    let second = resources.prepare(&device, &queue, &streamer, &frame, true);
    assert_eq!(second.active_probe_count, 1);
    assert_eq!(second.uploaded_cubemap_count, 0);
    assert_eq!(second.rejected_cubemap_count, 0);

    let disabled = resources.prepare(&device, &queue, &streamer, &frame, false);
    assert_eq!(disabled.extracted_probe_count, 1);
    assert_eq!(disabled.active_probe_count, 0);
    assert_eq!(disabled.uploaded_cubemap_count, 0);
    assert_eq!(disabled.rejected_cubemap_count, 0);

    let validation_error = pollster::block_on(error_scope.pop());
    assert!(
        validation_error.is_none(),
        "probe texture upload and bindings should pass WGPU validation: {validation_error:?}"
    );
}

#[test]
fn environment_preview_defers_local_provider_resources_until_a_probe_is_enabled() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
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
    let disabled = resources.prepare(&device, &queue, &streamer, &frame, false);
    assert_eq!(disabled.active_probe_count, 0);
    assert!(resources.is_environment_only_placeholder_for_tests());

    let enabled = resources.prepare(&device, &queue, &streamer, &frame, true);
    assert_eq!(enabled.active_probe_count, 1);
    assert_eq!(enabled.uploaded_cubemap_count, 1);
    assert!(!resources.is_environment_only_placeholder_for_tests());
    assert!(resources.requires_generic_environment_pbr());

    let hidden = resources.prepare(&device, &queue, &streamer, &frame, false);
    assert_eq!(hidden.active_probe_count, 0);
    assert!(
        resources.requires_generic_environment_pbr(),
        "a provider upgrade remains sticky so Base variants cannot thrash between environment ABIs"
    );
}

#[test]
fn full_scene_reflection_resources_do_not_report_an_environment_preview_upgrade() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, .. } = backend;
    let resources = SceneReflectionProbeResources::new(&device);

    assert!(
        !resources.requires_generic_environment_pbr(),
        "full-scene resource capacity is not an environment-preview provider upgrade"
    );
}

#[test]
fn environment_preview_placeholder_satisfies_the_local_provider_binding_abi() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
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
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = texture_bind_group_layout(&device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let streamer = ResourceStreamer::new_for_test(asset_manager, &device, &queue, &texture_layout);
    let capture_target = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://render-targets/environment-preview-planar-capture.ztexture",
    ));
    let mut frame = probe_frame(ResourceId::from_stable_label("builtin://unused-probe"));
    frame.extract.environment.probes.clear();
    frame
        .extract
        .view
        .selected_camera_descriptor_mut()
        .expect("probe fixture has a selected camera")
        .target = RenderCameraTarget::Texture(capture_target.clone());
    frame.extract.lighting.advanced_lighting.planar_probes = vec![PlanarReflectionProbeData {
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
    let report = resources.prepare(&device, &queue, &streamer, &frame, false);
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
fn render_probe_resources_report_source_cubemap_rejection() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let RenderBackend { device, queue, .. } = backend;
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

    let report = resources.prepare(&device, &queue, &streamer, &probe_frame(cubemap), true);

    assert_eq!(report.active_probe_count, 0);
    assert_eq!(report.uploaded_cubemap_count, 0);
    assert_eq!(report.rejected_cubemap_count, 1);
    assert_eq!(
        report
            .first_rejection
            .expect("typed rejection should be retained")
            .reason,
        ReflectionProbeAssetRejectionReason::SourceCubemapRequiresPrefiltering
    );
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
