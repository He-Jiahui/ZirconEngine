use std::sync::Arc;

use crate::asset::{
    texture_asset_from_ibl_bake_artifact_pmrem, AssetUri, ProjectAssetManager, TextureAsset,
};
use crate::core::framework::render::{
    build_source_cubemap_from_equirect, IblBakeArtifactBlob, IblBakeArtifactContents,
    IblBakeArtifactDescriptor, IblBakeArtifactPayload, ProbeInfluenceShape, ProceduralSkyParams,
    ReflectionProbeData,
};
use crate::core::math::{Quat, UVec2, Vec3};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;
use crate::scene::world::World;

use super::super::resources::{
    SceneReflectionProbeResources, MAX_REFLECTION_PROBES, REFLECTION_PROBE_FACE_SIZE,
    REFLECTION_PROBE_MIP_COUNT,
};
use super::super::upload::ReflectionProbeAssetRejectionReason;

#[test]
fn render_probe_gpu_capacity_matches_plan_v1_limit() {
    assert_eq!(MAX_REFLECTION_PROBES, 64);
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

    let first = resources.prepare(&queue, &streamer, &frame, true);
    assert_eq!(first.extracted_probe_count, 1);
    assert_eq!(first.active_probe_count, 1);
    assert_eq!(first.uploaded_cubemap_count, 1);
    assert_eq!(first.rejected_cubemap_count, 0);
    assert_eq!(first.first_rejection, None);

    let second = resources.prepare(&queue, &streamer, &frame, true);
    assert_eq!(second.active_probe_count, 1);
    assert_eq!(second.uploaded_cubemap_count, 0);
    assert_eq!(second.rejected_cubemap_count, 0);

    let disabled = resources.prepare(&queue, &streamer, &frame, false);
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

    let report = resources.prepare(&queue, &streamer, &probe_frame(cubemap), true);

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
