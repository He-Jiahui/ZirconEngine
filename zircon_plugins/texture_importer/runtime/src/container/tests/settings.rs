use super::common::*;
use zircon_runtime::asset::ImportedAsset;
use zircon_runtime::asset::TexturePayload;

#[test]
fn container_importer_applies_descriptor_settings_without_expanding_payload() {
    let imported = import_container_fixture_with_settings(
        "albedo.dds",
        tiny_dds_bytes(),
        r#"
texture_format = "rgba16float"
sampler = "nearest"
asset_usage = "render_world"
"#,
    );

    match imported {
        ImportedAsset::Texture(texture) => {
            assert!(texture.rgba.is_empty());
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "rgba16float");
            assert_eq!(descriptor.depth_or_array_layers, 1);
            assert_eq!(
                descriptor.sampler.mag_filter,
                zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
            );
            assert_eq!(
                descriptor.sampler.min_filter,
                zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
            );
            assert_eq!(
                descriptor.asset_usage,
                vec![zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld]
            );
            match texture.payload {
                TexturePayload::Container {
                    format,
                    mip_count,
                    array_layers,
                    ..
                } => {
                    assert_eq!(format, "dds/DXT1");
                    assert_eq!(mip_count, 3);
                    assert_eq!(array_layers, 1);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn container_importer_rejects_array_layout_without_decoded_rgba() {
    let error = import_container_error_with_settings(
        "albedo.dds",
        tiny_dds_bytes(),
        r#"
[array_layout]
row_count = 2
"#,
    );

    assert!(
        error.contains("texture import setting `array_layout` requires a decoded rgba8 image"),
        "unexpected error: {error}"
    );
}
