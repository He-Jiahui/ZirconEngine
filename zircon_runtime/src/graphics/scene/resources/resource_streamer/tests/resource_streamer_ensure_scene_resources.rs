use crate::core::framework::render::{
    RenderCameraTargetWritebackStatus, RenderColorLookupSettings, RenderColorLookupTextureLayout,
    RenderFrameExtract, RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension,
    RenderImageFallbackKind, RenderImageUsage, RenderPostProcessEffectStackSettings,
    RenderSamplerDescriptor, RenderWorldSnapshotHandle, TextureMetadata,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::types::{
    ViewportRenderFrame, ViewportRenderOutputTarget, FRAMEWORK_OUTPUT_FORMAT_LABEL,
    LINEAR_OUTPUT_FORMAT_LABEL,
};
use crate::scene::World;

use super::{
    direct_submission_output_target_writeback_plan, effect_stack_lut_texture_id,
    effect_stack_lut_texture_request, effect_stack_lut_texture_status,
    output_target_graph_import_report, output_target_writeback_plan, EffectStackLutTextureStatus,
};

#[test]
fn scene_resource_prepare_deduplicates_instance_level_asset_ensures() {
    let source = include_str!("../resource_streamer_ensure_scene_resources.rs");

    for declaration in [
        ["let mut direct_mesh", "_readiness = HashMap::new()"].concat(),
        ["let mut ensured_", "materials = HashSet::new()"].concat(),
        ["let mut sprite_texture", "_readiness = HashMap::new()"].concat(),
    ] {
        assert!(source.contains(&declaration), "missing {declaration}");
    }
}

#[test]
fn scene_resource_prepare_collects_mip_streaming_after_material_preparation() {
    let source = include_str!("../resource_streamer_ensure_scene_resources.rs");
    let material_prepare = source
        .find("self.ensure_material")
        .expect("scene preparation ensures materials");
    let visibility_collect = source
        .find("self.collect_texture_mip_streaming_visibility(frame)")
        .expect("scene preparation collects texture streaming visibility");
    let streaming_apply = source
        .find("self.apply_texture_mip_streaming(")
        .expect("scene preparation applies texture streaming before draw preparation");

    assert!(material_prepare < visibility_collect);
    assert!(visibility_collect < streaming_apply);
}

#[test]
fn scene_resource_prepare_routes_texture_producers_into_the_frame_transaction() {
    let production = include_str!("../resource_streamer_ensure_scene_resources.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene resource preparation test boundary");

    assert!(production.contains("ensure_material_for_frame("));
    assert!(production.contains("ensure_texture_for_frame("));
    assert!(production.contains("ensure_sprite_texture_for_frame("));
    assert!(production.contains("ensure_irradiance_volume_texture("));
    assert!(production.contains("ensure_post_process_lut_texture_snapshot("));
    assert!(production.contains("load_texture_asset_snapshot(texture_id)"));
    assert!(!production.contains("load_texture_asset(texture_id)"));
    assert!(production.contains("submission_transaction"));
    assert!(!production.contains("self.ensure_material("));
    assert!(!production.contains("self.ensure_texture("));
    assert!(!production.contains("self.ensure_sprite_texture("));
}

#[test]
fn product_scene_resource_prepare_does_not_receive_queue_authority() {
    let production = include_str!("../resource_streamer_ensure_scene_resources.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene resource preparation test boundary");
    let material = include_str!("../resource_streamer_ensure_material.rs");
    let frame_material = material
        .split("pub(crate) fn ensure_material_for_frame(")
        .nth(1)
        .and_then(|source| source.split("fn ensure_material_internal(").next())
        .expect("frame material preparation boundary");
    let internal_material = material
        .split("fn ensure_material_internal(")
        .nth(1)
        .and_then(|source| source.split("crate::profile_scope!").next())
        .expect("internal material preparation signature");

    assert!(!production.contains("queue: &wgpu::Queue"));
    assert!(!frame_material.contains("wgpu::Queue"));
    assert!(!internal_material.contains("wgpu::Queue"));
}

#[test]
fn effect_stack_lut_texture_id_uses_enabled_lookup_handle() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "postprocess/lut/filmic",
    ));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
        color_lookup: RenderColorLookupSettings {
            texture: Some(texture),
            intensity: 0.75,
            ..Default::default()
        },
        ..Default::default()
    };
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    assert_eq!(effect_stack_lut_texture_id(&frame), Some(texture.id()));
}

#[test]
fn effect_stack_lut_texture_id_ignores_disabled_lookup_handle() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "postprocess/lut/disabled",
    ));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
        color_lookup: RenderColorLookupSettings {
            texture: Some(texture),
            intensity: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    assert_eq!(effect_stack_lut_texture_id(&frame), None);
}

#[test]
fn effect_stack_lut_texture_request_tracks_enabled_lut_without_handle() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
        color_lookup: RenderColorLookupSettings {
            texture: None,
            intensity: 0.5,
            ..Default::default()
        },
        ..Default::default()
    };
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    assert!(effect_stack_lut_texture_request(&frame).is_some());
    assert_eq!(effect_stack_lut_texture_id(&frame), None);
}

#[test]
fn effect_stack_lut_texture_status_accepts_2d_strip_for_current_binding() {
    let descriptor = texture_descriptor(33 * 33, 33, 1, RenderImageDimension::D2);

    assert_eq!(
        effect_stack_lut_texture_status(
            RenderColorLookupTextureLayout::Texture2dStrip { size: 33 },
            &descriptor,
        ),
        EffectStackLutTextureStatus::Ready2dStrip
    );
}

#[test]
fn effect_stack_lut_texture_status_accepts_3d_lut_for_texture_3d_binding() {
    let descriptor = texture_descriptor(33, 33, 33, RenderImageDimension::D3);

    assert_eq!(
        effect_stack_lut_texture_status(
            RenderColorLookupTextureLayout::Texture3d { size: 33 },
            &descriptor,
        ),
        EffectStackLutTextureStatus::Ready3d
    );
}

#[test]
fn effect_stack_lut_texture_status_rejects_non_2d_binding_shapes() {
    let array_descriptor = texture_descriptor(64, 64, 4, RenderImageDimension::D2);
    let wrong_strip = texture_descriptor(64, 64, 1, RenderImageDimension::D2);

    assert_eq!(
        effect_stack_lut_texture_status(RenderColorLookupTextureLayout::Auto, &array_descriptor),
        EffectStackLutTextureStatus::UnsupportedShape
    );
    assert_eq!(
        effect_stack_lut_texture_status(
            RenderColorLookupTextureLayout::Texture2dStrip { size: 33 },
            &wrong_strip,
        ),
        EffectStackLutTextureStatus::UnsupportedShape
    );
}

#[test]
fn output_target_graph_import_report_marks_srgb_texture_ready_for_direct_import() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/output-target/graph-import/srgb",
    ));
    let frame = ViewportRenderFrame::from_extract(
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        UVec2::new(64, 64),
    )
    .with_output_target(ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(64, 64),
        format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
    });

    let report = output_target_graph_import_report(
        &frame
            .output_target()
            .graph_import_plan(Some("rgba8unorm_srgb")),
    );

    assert_eq!(
        report.status,
        crate::core::framework::render::RenderCameraTargetGraphImportStatus::ReadyForDirectImport
    );
    assert_eq!(report.target_size, UVec2::new(64, 64));
    assert_eq!(report.direct_import_count, 0);
    assert_eq!(report.conversion_writeback_count, 0);
    assert_eq!(report.blocked_count, 0);
    assert_eq!(
        output_target_writeback_plan(report).status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        direct_submission_output_target_writeback_plan(report).status,
        RenderCameraTargetWritebackStatus::ReadyForCopy
    );
}

#[test]
fn output_target_graph_import_report_keeps_linear_texture_on_writeback_path() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/output-target/graph-import/linear",
    ));
    let frame = ViewportRenderFrame::from_extract(
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        UVec2::new(64, 64),
    )
    .with_output_target(ViewportRenderOutputTarget::Texture {
        handle: texture,
        size: UVec2::new(64, 64),
        format: LINEAR_OUTPUT_FORMAT_LABEL,
    });

    let report = output_target_graph_import_report(
        &frame.output_target().graph_import_plan(Some("rgba8unorm")),
    );

    assert_eq!(
        report.status,
        crate::core::framework::render::RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(report.direct_import_count, 0);
    assert_eq!(report.conversion_writeback_count, 1);
    assert_eq!(report.blocked_count, 0);
    assert_eq!(
        output_target_writeback_plan(report).status,
        RenderCameraTargetWritebackStatus::ReadyForConversion
    );
    assert_eq!(
        direct_submission_output_target_writeback_plan(report).status,
        RenderCameraTargetWritebackStatus::ReadyForConversion
    );
}

fn texture_descriptor(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    dimension: RenderImageDimension,
) -> RenderImageDescriptor {
    RenderImageDescriptor {
        width,
        height,
        depth_or_array_layers,
        dimension,
        format: "rgba8unorm".to_string(),
        color_space: RenderImageColorSpace::Linear,
        metadata: TextureMetadata {
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        },
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![RenderImageUsage::Sampled],
        asset_usage: Vec::new(),
        mip_count: 1,
        fallback: RenderImageFallbackKind::MissingImage,
    }
}
