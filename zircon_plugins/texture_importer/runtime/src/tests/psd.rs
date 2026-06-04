use crate::plugin_registration;

use super::support::tiny_psd_bytes;

#[test]
fn psd_importer_decodes_flattened_rgba_texture_asset() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("swatch.psd"))
        .unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        "swatch.psd".into(),
        zircon_runtime::asset::AssetUri::parse("res://textures/swatch.psd").unwrap(),
        tiny_psd_bytes(),
        Default::default(),
    );

    let outcome = importer.import(&context).unwrap();
    let imported = &outcome
        .root_entry()
        .expect("root texture asset entry")
        .asset;

    match imported {
        zircon_runtime::asset::ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba, vec![12, 34, 56, 200]);
            assert_eq!(
                texture.payload,
                zircon_runtime::asset::TexturePayload::Rgba8
            );
            assert!(texture.descriptor.is_some());
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn psd_importer_applies_texture_descriptor_settings() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("swatch.psd"))
        .unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        "swatch.psd".into(),
        zircon_runtime::asset::AssetUri::parse("res://textures/swatch.psd").unwrap(),
        tiny_psd_bytes(),
        r#"
texture_format = "rgba16float"
is_srgb = false
sampler = "nearest"
asset_usage = "render_world"
"#
        .parse()
        .expect("valid texture import settings"),
    );

    let outcome = importer.import(&context).unwrap();
    let imported = &outcome
        .root_entry()
        .expect("root texture asset entry")
        .asset;

    match imported {
        zircon_runtime::asset::ImportedAsset::Texture(texture) => {
            assert_eq!(texture.rgba, vec![12, 34, 56, 200]);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "rgba16float");
            assert_eq!(
                descriptor.color_space,
                zircon_runtime::core::framework::render::RenderImageColorSpace::Linear
            );
            assert_eq!(
                descriptor.sampler.mag_filter,
                zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
            );
            assert_eq!(
                descriptor.sampler.min_filter,
                zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
            );
            assert_eq!(
                descriptor.sampler.mipmap_filter,
                zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
            );
            assert_eq!(
                descriptor.asset_usage,
                vec![zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld]
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}
