use super::*;

#[test]
fn render_asset_usage_alias_accepts_single_token() {
    let settings = r#"render_asset_usage = "gpu""#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid render asset usage alias");

    assert_eq!(
        descriptor.asset_usage,
        vec![RenderImageAssetUsage::RenderWorld]
    );
}

#[test]
fn depth_or_array_layers_updates_array_layer_count_for_2d_arrays() {
    let settings = r#"depth_or_array_layers = 4"#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid depth override");

    assert_eq!(descriptor.depth_or_array_layers, 4);
    assert_eq!(descriptor.array_layer_count, 4);
}

#[test]
fn array_layer_count_updates_depth_or_array_layers_for_2d_arrays() {
    let settings = r#"array_layer_count = 3"#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid array layer override");

    assert_eq!(descriptor.depth_or_array_layers, 3);
    assert_eq!(descriptor.array_layer_count, 3);
}

#[test]
fn mismatched_2d_extent_settings_report_error() {
    let settings = r#"
array_layer_count = 2
depth_or_array_layers = 4
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("mismatched extent settings");

    assert!(matches!(
        error,
        TextureDescriptorError::MismatchedExtentSettings {
            array_key: "array_layer_count",
            depth_key: "depth_or_array_layers",
        }
    ));
    assert!(
        error.to_string().contains(
            "texture import settings `array_layer_count` and `depth_or_array_layers` must match for 1d/2d array textures"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn dimension_3d_rejects_multiple_array_layers() {
    let settings = r#"
dimension = "3d"
array_layers = 2
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("3d array layer override");

    assert!(
        error
            .to_string()
            .contains("texture import setting `array_layers` must be 1 for 3d textures"),
        "unexpected error: {error}"
    );
}

#[test]
fn dimension_3d_keeps_depth_and_single_array_layer() {
    let settings = r#"
dimension = "3d"
depth = 4
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid 3d depth override");

    assert_eq!(descriptor.dimension, RenderImageDimension::D3);
    assert_eq!(descriptor.depth_or_array_layers, 4);
    assert_eq!(descriptor.array_layer_count, 1);
}

#[test]
fn dimension_cube_defaults_to_six_faces() {
    let settings = r#"dimension = "cube""#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid cube dimension");

    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.depth_or_array_layers, 6);
    assert_eq!(descriptor.array_layer_count, 6);
}

#[test]
fn dimension_cubemap_alias_requires_face_multiple_layers() {
    let settings = r#"
dimension = "cubemap"
array_layers = 5
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("invalid cube face count");

    assert!(
        error
            .to_string()
            .contains("cube texture layer count must be a non-zero multiple of six faces, found 5"),
        "unexpected error: {error}"
    );
}

#[test]
fn import_extent_override_replaces_existing_2d_container_layers() {
    let settings = r#"depth_or_array_layers = 4"#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::container("dds/DXT1", 1, 12)
        .apply_import_settings(&settings)
        .expect("valid depth override");

    assert_eq!(descriptor.depth_or_array_layers, 4);
    assert_eq!(descriptor.array_layer_count, 4);
}

#[test]
fn bevy_alias_diagnostics_report_actual_setting_keys() {
    let cases = [
        (
            r#"texture_format = 1"#,
            "texture import setting `texture_format` must be a string",
        ),
        (
            r#"is_srgb = "false""#,
            "texture import setting `is_srgb` must be a boolean",
        ),
        (
            r#"sampler = 1"#,
            "texture import setting `sampler` must be a table or string",
        ),
        (
            r#"render_asset_usage = 1"#,
            "texture import setting `render_asset_usage` must be a string or array of strings",
        ),
        (
            r#"render_asset_usage = "video_memory""#,
            "unsupported texture render_asset_usage `video_memory`",
        ),
    ];

    for (settings, expected) in cases {
        let settings = settings.parse::<toml::Table>().expect("valid toml");
        let error = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect_err("invalid alias setting");

        assert!(
            error.to_string().contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn invalid_import_settings_report_typed_error_variants() {
    let settings = r#"sampler = 1"#.parse::<toml::Table>().expect("valid toml");
    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("invalid sampler setting");

    assert!(matches!(
        error,
        TextureDescriptorError::SettingType {
            ref name,
            expected: "a table or string",
        } if name == "sampler"
    ));

    let settings = r#"render_asset_usage = "video_memory""#
        .parse::<toml::Table>()
        .expect("valid toml");
    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("unsupported render asset usage");

    assert!(matches!(
        error,
        TextureDescriptorError::UnsupportedToken {
            ref kind,
            ref value,
        } if kind == "render_asset_usage" && value == "video_memory"
    ));
}

#[test]
fn linear_color_space_normalizes_default_rgba8_format_to_linear() {
    let settings = r#"color_space = "linear""#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid linear color space");

    assert_eq!(descriptor.format, RGBA8_UNORM_FORMAT);
    assert_eq!(
        descriptor.to_render_image_descriptor(2, 2).format,
        RGBA8_UNORM_FORMAT
    );
}

#[test]
fn srgb_color_space_normalizes_linear_rgba8_format_to_srgb() {
    let settings = r#"
format = "rgba8unorm"
color_space = "srgb"
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid srgb color space");

    assert_eq!(descriptor.format, RGBA8_UNORM_SRGB_FORMAT);
    assert_eq!(
        descriptor.to_render_image_descriptor(2, 2).format,
        RGBA8_UNORM_SRGB_FORMAT
    );
}

#[test]
fn unknown_color_space_is_rejected_by_the_import_contract() {
    let settings = r#"color_space = "unknown""#.parse::<toml::Table>().expect("valid toml");

    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("unknown color spaces are not valid texture metadata");

    assert!(matches!(
        error,
        TextureDescriptorError::UnsupportedToken {
            ref kind,
            ref value,
        } if kind == "color_space" && value == "unknown"
    ));
}

#[test]
fn import_color_space_is_written_to_texture_metadata() {
    let settings = r#"color_space = "linear""#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid linear color space");

    assert_eq!(
        descriptor.metadata.color_space,
        RenderImageColorSpace::Linear
    );
    assert_eq!(
        descriptor.to_render_image_descriptor(2, 2).color_space,
        RenderImageColorSpace::Linear
    );
}

#[test]
fn render_image_descriptor_preserves_texture_metadata() {
    let settings = r#"
usage_hint = "normal"
mip_policy = "generate_offline"
normal_convention = "dx"
compression = "bc5"
mip_filter = "box"
mip_bias = 0.5
max_anisotropy = 8
streaming_enabled = false
"#
    .parse::<toml::Table>()
    .expect("valid texture metadata settings");
    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid descriptor metadata");

    let render_descriptor = descriptor.to_render_image_descriptor(4, 4);

    assert_eq!(render_descriptor.metadata, descriptor.metadata);
    assert!(!render_descriptor.metadata.streaming_enabled);
}

#[test]
fn import_settings_control_texture_mip_streaming_and_reject_non_boolean_values() {
    let disabled_settings = r#"streaming_enabled = false"#
        .parse::<toml::Table>()
        .expect("valid streaming setting");
    let disabled = TextureAssetDescriptor::default()
        .apply_import_settings(&disabled_settings)
        .expect("boolean streaming setting is valid");

    assert!(!disabled.metadata.streaming_enabled);
    assert!(TextureAssetDescriptor::default().metadata.streaming_enabled);

    let invalid_settings = r#"streaming_enabled = "false""#
        .parse::<toml::Table>()
        .expect("valid toml with an invalid setting type");
    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&invalid_settings)
        .expect_err("streaming_enabled must be boolean");

    assert!(matches!(
        error,
        TextureDescriptorError::SettingType { ref name, .. } if name == "streaming_enabled"
    ));
}

#[test]
fn import_settings_parse_texture_metadata_tokens() {
    let settings = r#"
usage_hint = "normal"
mip_policy = "generate_offline"
normal_convention = "dx"
compression = "bc5"
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("valid texture metadata");

    assert_eq!(descriptor.metadata.usage_hint, TextureUsageHint::Normal);
    assert_eq!(
        descriptor.metadata.mip_policy,
        TextureMipPolicy::GenerateOffline
    );
    assert_eq!(
        descriptor.metadata.normal_convention,
        TextureNormalConvention::TangentSpaceDx
    );
    assert_eq!(
        descriptor.metadata.compression,
        TextureCompressionTarget::Bc5
    );
    assert_eq!(descriptor.metadata.mip_filter, TextureMipFilter::Box);
    assert_eq!(descriptor.metadata.mip_bias, 0.5);
    assert_eq!(descriptor.metadata.max_anisotropy, 8);
    assert_eq!(
        descriptor.metadata.color_space,
        RenderImageColorSpace::Linear
    );
}

#[test]
fn usage_hint_selects_color_space_when_not_explicitly_overridden() {
    let normal_settings =
        r#"usage_hint = "normal""#.parse::<toml::Table>().expect("valid normal settings");
    let ui_settings = r#"usage_hint = "ui""#.parse::<toml::Table>().expect("valid ui settings");
    let hdr_settings = r#"usage_hint = "hdr""#.parse::<toml::Table>().expect("valid hdr settings");

    let normal = TextureAssetDescriptor::default()
        .apply_import_settings(&normal_settings)
        .expect("normal defaults should be valid");
    let ui = TextureAssetDescriptor::default()
        .apply_import_settings(&ui_settings)
        .expect("ui defaults should be valid");
    let hdr = TextureAssetDescriptor::default()
        .apply_import_settings(&hdr_settings)
        .expect("hdr defaults should be valid");
    let albedo = TextureAssetDescriptor::default()
        .apply_import_settings(&toml::Table::new())
        .expect("albedo defaults should be valid");

    assert_eq!(normal.color_space, RenderImageColorSpace::Linear);
    assert_eq!(normal.metadata.color_space, RenderImageColorSpace::Linear);
    assert_eq!(
        normal.metadata.mip_policy,
        TextureMipPolicy::GenerateOffline
    );
    assert_eq!(normal.metadata.mip_filter, TextureMipFilter::Box);
    assert_eq!(normal.metadata.compression, TextureCompressionTarget::Bc5);
    assert_eq!(
        normal.metadata.normal_convention,
        TextureNormalConvention::TangentSpaceDx
    );
    assert_eq!(normal.format, RGBA8_UNORM_FORMAT);
    assert_eq!(ui.color_space, RenderImageColorSpace::Srgb);
    assert_eq!(ui.metadata.color_space, RenderImageColorSpace::Srgb);
    assert_eq!(ui.metadata.mip_policy, TextureMipPolicy::GenerateOffline);
    assert_eq!(ui.metadata.mip_filter, TextureMipFilter::Box);
    assert_eq!(
        ui.metadata.compression,
        TextureCompressionTarget::Uncompressed
    );
    assert_eq!(hdr.metadata.compression, TextureCompressionTarget::Bc6h);
    assert_eq!(hdr.metadata.mip_policy, TextureMipPolicy::GenerateOffline);
    assert_eq!(albedo.metadata.compression, TextureCompressionTarget::Bc7);
    assert_eq!(
        albedo.metadata.mip_policy,
        TextureMipPolicy::GenerateOffline
    );
}

#[test]
fn container_format_preserves_from_source_mip_policy_without_an_override() {
    let descriptor = TextureAssetDescriptor::container("dds/ati2", 5, 1)
        .apply_import_settings(&toml::Table::new())
        .expect("container defaults should be valid");

    assert_eq!(descriptor.metadata.mip_policy, TextureMipPolicy::FromSource);
}

#[test]
fn uncompressed_container_mip_chain_preserves_from_source_mip_policy() {
    let descriptor = TextureAssetDescriptor::container(RGBA8_UNORM_SRGB_FORMAT, 5, 1)
        .apply_import_settings(&toml::Table::new())
        .expect("uncompressed container defaults should be valid");

    assert_eq!(descriptor.metadata.mip_policy, TextureMipPolicy::FromSource);
}

#[test]
fn embedded_mip_chain_warns_when_offline_generation_is_requested() {
    let settings = r#"mip_policy = "generate_offline""#
        .parse::<toml::Table>()
        .expect("valid mip policy settings");
    let descriptor = TextureAssetDescriptor::container("dds/DXT1", 3, 1)
        .apply_import_settings(&settings)
        .expect("valid container descriptor");

    assert!(descriptor
        .validate_metadata("textures/albedo.dds")
        .iter()
        .any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning
                && diagnostic.message
                    == "'textures/albedo.dds' already contains 3 mips; falling back to from_source"
        }));
}

#[test]
fn descriptor_metadata_validation_uses_the_canonical_metadata_field() {
    let mut descriptor = TextureAssetDescriptor::default();
    descriptor.metadata.usage_hint = TextureUsageHint::Normal;

    assert!(descriptor
        .validate_metadata("textures/normal.png")
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error));
}

#[test]
fn runtime_mip_generation_rejects_non_2d_dimensions_before_gpu_upload() {
    let mut descriptor = TextureAssetDescriptor::default();
    descriptor.dimension = RenderImageDimension::D3;
    descriptor.mip_count = 3;
    descriptor.metadata.mip_policy = TextureMipPolicy::GenerateRuntime;
    descriptor.metadata.mip_filter = TextureMipFilter::Box;

    assert!(descriptor
        .validate_metadata("textures/volume.texture")
        .iter()
        .any(|diagnostic| {
            diagnostic.severity == TextureMetadataDiagnosticSeverity::Error
                && diagnostic.message.contains("only 2d or cube")
        }));
}
