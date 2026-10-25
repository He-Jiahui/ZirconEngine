//! Memory-only source values; no imported/project asset, provider or device is fabricated.

use zircon_runtime::asset::{
    AssetUri, TextureAsset, TexturePayload, TextureUploadReadiness, TextureUploadSupport,
};
use zircon_runtime::core::framework::render::{
    RenderImageColorSpace, RenderImageDimension, TextureMetadataDiagnosticSeverity,
    TextureUsageHint,
};

use super::TextureSourceAdapter;

fn rgba() -> TextureAsset {
    TextureAsset::new_rgba8(
        AssetUri::parse("res://纹理/unit-source.png").unwrap(),
        2,
        1,
        vec![255, 0, 128, 255, 0, 255, 64, 0],
    )
}

#[test]
fn texture_source_borrows_original_payload_and_preserves_complete_descriptor() {
    let asset = rgba();
    let before = asset.clone();
    let view = TextureSourceAdapter::read(&asset);
    assert!(std::ptr::eq(view.asset, &asset));
    assert!(std::ptr::eq(
        view.active_payload_bytes(),
        asset.rgba.as_slice()
    ));
    assert_eq!(view.active_payload_bytes(), before.rgba.as_slice());
    assert_eq!(view.retained_payload_byte_len(), Some(8));
    assert_eq!(view.asset.uri, before.uri);
    assert_eq!(view.asset.descriptor, before.descriptor);
    assert_eq!(view.effective_descriptor, asset.render_image_descriptor());
    assert_eq!(view.descriptor_normalization_changed, Some(false));
    assert_eq!(asset, before);
}

#[test]
fn texture_source_absent_authored_descriptor_is_distinct_from_no_diagnostics() {
    let authored = rgba();
    let authored_view = TextureSourceAdapter::read(&authored);
    assert!(authored_view.authored_metadata_diagnostics.is_some());
    let mut legacy = authored.clone();
    legacy.descriptor = None;
    let before = legacy.clone();
    let view = TextureSourceAdapter::read(&legacy);
    assert!(view.asset.descriptor.is_none());
    assert!(view.authored_metadata_diagnostics.is_none());
    assert_eq!(view.descriptor_normalization_changed, None);
    assert_eq!(view.effective_descriptor, legacy.render_image_descriptor());
    assert_eq!(legacy, before);
}

#[test]
fn texture_source_raw_invalid_descriptor_is_not_replaced_by_normalized_defaults() {
    let mut asset = rgba();
    let descriptor = asset.descriptor.as_mut().unwrap();
    descriptor.format = " ".into();
    descriptor.mip_count = 0;
    descriptor.depth_or_array_layers = 0;
    descriptor.metadata.mip_bias = -0.0;
    let before = asset.clone();
    let view = TextureSourceAdapter::read(&asset);
    let raw = view.asset.descriptor.as_ref().unwrap();
    assert_eq!(raw.format, " ");
    assert_eq!(raw.mip_count, 0);
    assert_eq!(raw.depth_or_array_layers, 0);
    assert_eq!(raw.metadata.mip_bias.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(view.descriptor_normalization_changed, Some(true));
    assert_eq!(view.effective_descriptor.mip_count, 1);
    assert_eq!(view.effective_descriptor.depth_or_array_layers, 1);
    assert_ne!(view.effective_descriptor.format, raw.format);
    assert_eq!(
        view.authored_metadata_diagnostics,
        Some(raw.validate_metadata(&asset.uri.to_string()))
    );
    assert_eq!(asset, before);
}

#[test]
fn texture_source_raw_normal_map_diagnostics_survive_color_space_normalization() {
    let mut asset = rgba();
    let descriptor = asset.descriptor.as_mut().unwrap();
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor.metadata.usage_hint = TextureUsageHint::Normal;
    descriptor.metadata.color_space = RenderImageColorSpace::Srgb;
    let before = asset.clone();
    let view = TextureSourceAdapter::read(&asset);
    assert_eq!(
        view.effective_descriptor.metadata.color_space,
        RenderImageColorSpace::Linear
    );
    assert_eq!(
        view.asset.descriptor.as_ref().unwrap().metadata.color_space,
        RenderImageColorSpace::Srgb
    );
    assert!(view
        .authored_metadata_diagnostics
        .as_ref()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
    assert_eq!(asset, before);
}

#[test]
fn texture_source_container_keeps_inactive_rgba_and_independent_payload_counts() {
    let mut asset = TextureAsset::new_container(
        AssetUri::parse("res://texture-unit-source.container").unwrap(),
        8,
        4,
        "opaque/container",
        vec![1, 2, 3, 4, 5],
        5,
        7,
    );
    asset.rgba = vec![9, 8, 7];
    let descriptor = asset.descriptor.as_mut().unwrap();
    descriptor.mip_count = 3;
    descriptor.depth_or_array_layers = 2;
    let before = asset.clone();
    let view = TextureSourceAdapter::read(&asset);
    let TexturePayload::Container {
        bytes,
        mip_count,
        array_layers,
        format,
    } = &view.asset.payload
    else {
        panic!("the original container must remain a container");
    };
    assert!(std::ptr::eq(view.active_payload_bytes(), bytes.as_slice()));
    assert_eq!(view.active_payload_bytes(), [1, 2, 3, 4, 5]);
    assert_eq!(view.retained_payload_byte_len(), Some(8));
    assert_eq!((*mip_count, *array_layers), (5, 7));
    assert_eq!(format, "opaque/container");
    assert_eq!(view.asset.rgba, [9, 8, 7]);
    assert_eq!(
        (
            view.effective_descriptor.mip_count,
            view.effective_descriptor.depth_or_array_layers
        ),
        (3, 2)
    );
    assert_eq!(asset, before);
}

#[test]
fn texture_source_explicit_support_preflight_preserves_runtime_ready_and_unsupported_results() {
    let mut malformed = rgba();
    malformed.rgba.pop();
    let ready = rgba();
    for asset in [&ready, &malformed] {
        let before = (*asset).clone();
        let view = TextureSourceAdapter::read(asset);
        for support in [
            TextureUploadSupport::uncompressed_only(),
            TextureUploadSupport::all_compressed(),
        ] {
            assert_eq!(
                view.upload_preflight(support),
                asset.upload_readiness(support)
            );
        }
        assert_eq!(*asset, before);
    }
    assert!(matches!(
        TextureSourceAdapter::read(&ready)
            .upload_preflight(TextureUploadSupport::uncompressed_only()),
        TextureUploadReadiness::Ready { .. }
    ));
    assert!(matches!(
        TextureSourceAdapter::read(&malformed)
            .upload_preflight(TextureUploadSupport::uncompressed_only()),
        TextureUploadReadiness::Unsupported { .. }
    ));
}

#[test]
fn texture_source_array_cube_and_volume_dimensions_do_not_collapse_to_2d() {
    for (dimension, layers, bytes) in [
        (RenderImageDimension::D2, 2, 8),
        (RenderImageDimension::Cube, 6, 24),
        (RenderImageDimension::D3, 4, 16),
    ] {
        let mut asset = TextureAsset::new_rgba8(
            AssetUri::parse("res://texture-unit-shape").unwrap(),
            1,
            1,
            vec![255; bytes],
        );
        let descriptor = asset.descriptor.as_mut().unwrap();
        descriptor.dimension = dimension;
        descriptor.depth_or_array_layers = layers;
        let before = asset.clone();
        let view = TextureSourceAdapter::read(&asset);
        assert_eq!(view.asset.descriptor.as_ref().unwrap().dimension, dimension);
        assert_eq!(view.effective_descriptor.dimension, dimension);
        assert_eq!(view.effective_descriptor.depth_or_array_layers, layers);
        assert_eq!(
            view.upload_preflight(TextureUploadSupport::uncompressed_only()),
            asset.upload_readiness(TextureUploadSupport::uncompressed_only())
        );
        assert_eq!(asset, before);
    }
}

#[test]
fn texture_source_zero_extent_and_nonfinite_authoring_remain_inspectable() {
    let mut asset = rgba();
    asset.width = 0;
    asset.height = 0;
    asset.descriptor.as_mut().unwrap().metadata.mip_bias = f32::from_bits(0x7fc0_0001);
    let before = asset.clone();
    let view = TextureSourceAdapter::read(&asset);
    assert_eq!((view.asset.width, view.asset.height), (0, 0));
    assert_eq!(
        (
            view.effective_descriptor.width,
            view.effective_descriptor.height
        ),
        (0, 0)
    );
    assert_eq!(
        view.asset
            .descriptor
            .as_ref()
            .unwrap()
            .metadata
            .mip_bias
            .to_bits(),
        0x7fc0_0001
    );
    assert!(view
        .authored_metadata_diagnostics
        .as_ref()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
    assert_eq!(asset, before);
}
