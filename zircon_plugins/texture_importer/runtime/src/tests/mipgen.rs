use crate::importers::apply_texture_import_settings;
use zircon_runtime::asset::{AssetImportContext, AssetUri, TextureAsset};
use zircon_runtime::core::framework::render::TextureMipFilter;

fn import_context(settings: &str) -> AssetImportContext {
    AssetImportContext::new(
        "mip.png".into(),
        AssetUri::parse("res://textures/mip.png").expect("valid texture uri"),
        Vec::new(),
        settings.parse().expect("valid texture import settings"),
    )
}

#[test]
fn render_mipgen_offline_srgb_aware_average() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/mip.png").expect("valid texture uri"),
        2,
        2,
        vec![
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
        ],
    );
    let texture = apply_texture_import_settings(
        &import_context("mip_policy = \"generate_offline\""),
        texture,
    )
    .expect("offline mip generation should succeed");

    assert_eq!(texture.texture_descriptor().mip_count, 2);
    assert_eq!(
        texture.texture_descriptor().metadata.mip_filter,
        TextureMipFilter::Kaiser
    );
    assert_eq!(texture.rgba.len(), 20);
    assert_eq!(&texture.rgba[16..20], &[188, 188, 188, 255]);
}

#[test]
fn render_mipgen_decoded_images_default_to_an_offline_mip_chain() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/default-mips.png").expect("valid texture uri"),
        4,
        2,
        vec![64, 128, 192, 255].repeat(8),
    );
    let texture = apply_texture_import_settings(&import_context(""), texture)
        .expect("decoded image defaults should generate offline mips");

    assert_eq!(
        texture.texture_descriptor().metadata.mip_policy,
        zircon_runtime::core::framework::render::TextureMipPolicy::GenerateOffline
    );
    assert_eq!(texture.texture_descriptor().mip_count, 3);
    assert_eq!(texture.rgba.len(), (8 + 2 + 1) * 4);
}

#[test]
fn render_mipgen_offline_kaiser_chain_complete() {
    let mut pixels = Vec::with_capacity(256 * 256 * 4);
    for y in 0..256_u32 {
        for x in 0..256_u32 {
            pixels.extend([(x ^ y) as u8, x as u8, y as u8, 255]);
        }
    }
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/kaiser-chain.png").expect("valid texture uri"),
        256,
        256,
        pixels,
    );
    let texture = apply_texture_import_settings(
        &import_context("mip_policy = \"generate_offline\""),
        texture,
    )
    .expect("offline kaiser mip generation should succeed");
    let expected_len = (0..9)
        .map(|level| {
            let extent = (256_u32 >> level).max(1) as usize;
            extent * extent * 4
        })
        .sum::<usize>();

    assert_eq!(texture.texture_descriptor().mip_count, 9);
    assert_eq!(texture.rgba.len(), expected_len);
}

#[test]
fn render_mipgen_runtime_prepares_full_chain_without_cpu_mips() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/runtime-mips.png").expect("valid texture uri"),
        4,
        2,
        vec![32, 64, 96, 255].repeat(8),
    );
    let texture = apply_texture_import_settings(
        &import_context("mip_policy = \"generate_runtime\""),
        texture,
    )
    .expect("runtime mip preparation should succeed");

    assert_eq!(texture.texture_descriptor().mip_count, 3);
    assert_eq!(
        texture.texture_descriptor().metadata.mip_filter,
        TextureMipFilter::Box
    );
    assert_eq!(
        texture.texture_descriptor().metadata.compression,
        zircon_runtime::core::framework::render::TextureCompressionTarget::Uncompressed
    );
    assert_eq!(texture.rgba.len(), 4 * 2 * 4);
}

#[test]
fn render_mipgen_offline_normal_renormalized() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/normal.png").expect("valid texture uri"),
        2,
        2,
        vec![
            255, 128, 128, 255, 128, 255, 128, 255, 255, 128, 128, 255, 128, 255, 128, 255,
        ],
    );
    let texture = apply_texture_import_settings(
        &import_context(
            "usage_hint = \"normal\"\nmip_policy = \"generate_offline\"\ncompression = \"uncompressed\"",
        ),
        texture,
    )
    .expect("offline normal mip generation should succeed");

    let mip = &texture.rgba[16..20];
    let normal = [
        f32::from(mip[0]) / 127.5 - 1.0,
        f32::from(mip[1]) / 127.5 - 1.0,
        f32::from(mip[2]) / 127.5 - 1.0,
    ];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();

    assert_eq!(texture.texture_descriptor().mip_count, 2);
    assert!((length - 1.0).abs() <= 0.02, "normal length was {length}");
    assert_eq!(mip[3], 255);
}

#[test]
fn render_mipgen_offline_normal_mips_remain_unit_length() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/flat-normal.png").expect("valid texture uri"),
        4,
        4,
        vec![128, 128, 255, 255].repeat(16),
    );
    let texture = apply_texture_import_settings(
        &import_context(
            "usage_hint = \"normal\"\nmip_policy = \"generate_offline\"\ncompression = \"uncompressed\"",
        ),
        texture,
    )
    .expect("offline normal mip generation should succeed");

    let mut offset = 0;
    for extent in [4_usize, 2, 1] {
        for normal in texture.rgba[offset..offset + extent * extent * 4].chunks_exact(4) {
            let normal = [
                f32::from(normal[0]) / 127.5 - 1.0,
                f32::from(normal[1]) / 127.5 - 1.0,
                f32::from(normal[2]) / 127.5 - 1.0,
            ];
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            assert!(
                (length - 1.0).abs() <= 0.001,
                "normal length was {length} at packed byte offset {offset}"
            );
        }
        offset += extent * extent * 4;
    }

    assert_eq!(texture.texture_descriptor().mip_count, 3);
    assert_eq!(offset, texture.rgba.len());
}

#[test]
fn render_normal_import_transcodes_default_bc5_target() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/normal-bc5.png").expect("valid texture uri"),
        4,
        4,
        vec![128, 255, 0, 255].repeat(16),
    );
    let texture = apply_texture_import_settings(
        &import_context("usage_hint = \"normal\"\nmip_policy = \"generate_offline\""),
        texture,
    )
    .expect("normal import should transcode the bc5 target");

    assert!(texture.rgba.is_empty());
    assert_eq!(texture.texture_descriptor().format, "dds/ati2");
    let zircon_runtime::asset::TexturePayload::Container {
        bytes, mip_count, ..
    } = texture.payload
    else {
        panic!("normal bc5 target must use a dds container payload");
    };
    assert_eq!(mip_count, 3);
    assert_eq!(&bytes[..4], b"DDS ");
    assert_eq!(&bytes[84..88], b"ATI2");
    assert_eq!(bytes.len(), 128 + 3 * 16);
}
