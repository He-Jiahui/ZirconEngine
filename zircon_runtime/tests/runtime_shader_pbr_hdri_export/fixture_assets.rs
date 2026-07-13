use std::fs;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::project::{AssetMetaDocument, ProjectPaths};
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid, RGBA8_UNORM_FORMAT};

#[derive(Clone, Copy)]
pub(super) struct AmbientCgMetalFixture {
    pub(super) id: &'static str,
    pub(super) asset_dir: &'static str,
    pub(super) color: &'static str,
    pub(super) normal_gl: &'static str,
    pub(super) roughness: &'static str,
    pub(super) metalness: &'static str,
    pub(super) metallic_roughness: &'static str,
    pub(super) label_slug: &'static str,
}

pub(super) const AMBIENTCG_METAL009_COLOR: &str = "Metal009_1K-JPG_Color.jpg";
pub(super) const AMBIENTCG_METAL009_NORMAL_GL: &str = "Metal009_1K-JPG_NormalGL.jpg";
pub(super) const AMBIENTCG_METAL009_METALLIC_ROUGHNESS: &str =
    "Metal009_1K-JPG_MetallicRoughness.png";
pub(super) const AMBIENTCG_METAL008: AmbientCgMetalFixture = AmbientCgMetalFixture {
    id: "Metal008",
    asset_dir: "ambientcg_metal008_1k",
    color: "Metal008_1K-JPG_Color.jpg",
    normal_gl: "Metal008_1K-JPG_NormalGL.jpg",
    roughness: "Metal008_1K-JPG_Roughness.jpg",
    metalness: "Metal008_1K-JPG_Metalness.jpg",
    metallic_roughness: "Metal008_1K-JPG_MetallicRoughness.png",
    label_slug: "metal008",
};
pub(super) const AMBIENTCG_METAL025: AmbientCgMetalFixture = AmbientCgMetalFixture {
    id: "Metal025",
    asset_dir: "ambientcg_metal025_1k",
    color: "Metal025_1K-JPG_Color.jpg",
    normal_gl: "Metal025_1K-JPG_NormalGL.jpg",
    roughness: "Metal025_1K-JPG_Roughness.jpg",
    metalness: "Metal025_1K-JPG_Metalness.jpg",
    metallic_roughness: "Metal025_1K-JPG_MetallicRoughness.png",
    label_slug: "metal025",
};
pub(super) const AMBIENTCG_METAL029: AmbientCgMetalFixture = AmbientCgMetalFixture {
    id: "Metal029",
    asset_dir: "ambientcg_metal029_1k",
    color: "Metal029_1K-JPG_Color.jpg",
    normal_gl: "Metal029_1K-JPG_NormalGL.jpg",
    roughness: "Metal029_1K-JPG_Roughness.jpg",
    metalness: "Metal029_1K-JPG_Metalness.jpg",
    metallic_roughness: "Metal029_1K-JPG_MetallicRoughness.png",
    label_slug: "metal029",
};

const AMBIENTCG_METAL009_ASSET_DIR: &str = "ambientcg_metal009_1k";
const AMBIENTCG_METAL009_ROUGHNESS: &str = "Metal009_1K-JPG_Roughness.jpg";
const AMBIENTCG_METAL009_METALNESS: &str = "Metal009_1K-JPG_Metalness.jpg";
const AMBIENTCG_METAL009: AmbientCgMetalFixture = AmbientCgMetalFixture {
    id: "Metal009",
    asset_dir: AMBIENTCG_METAL009_ASSET_DIR,
    color: AMBIENTCG_METAL009_COLOR,
    normal_gl: AMBIENTCG_METAL009_NORMAL_GL,
    roughness: AMBIENTCG_METAL009_ROUGHNESS,
    metalness: AMBIENTCG_METAL009_METALNESS,
    metallic_roughness: AMBIENTCG_METAL009_METALLIC_ROUGHNESS,
    label_slug: "metal009",
};

pub(super) fn write_ambientcg_metal009_texture_assets(paths: &ProjectPaths) {
    write_ambientcg_metal_texture_assets(paths, AMBIENTCG_METAL009);
}

pub(super) fn write_ambientcg_metal_texture_assets(
    paths: &ProjectPaths,
    fixture: AmbientCgMetalFixture,
) {
    let source_dir = super::shader_test_asset_dir().join(fixture.asset_dir);
    let texture_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("textures")
        .join(fixture.asset_dir);
    fs::create_dir_all(&texture_dir).unwrap();

    for file_name in [
        fixture.color,
        fixture.normal_gl,
        fixture.roughness,
        fixture.metalness,
    ] {
        fs::copy(source_dir.join(file_name), texture_dir.join(file_name)).unwrap_or_else(|error| {
            panic!("copy ambientCG {} texture {file_name}: {error}", fixture.id)
        });
    }

    let metallic_roughness_path = texture_dir.join(fixture.metallic_roughness);
    pack_metallic_roughness_texture(
        &source_dir.join(fixture.roughness),
        &source_dir.join(fixture.metalness),
        &metallic_roughness_path,
    );
    write_linear_texture_meta(
        &texture_dir.join(fixture.normal_gl),
        &ambientcg_metal_texture_uri(fixture, fixture.normal_gl),
        &format!(
            "docs.tests.runtime.shader.ambientcg.{}.normal_gl",
            fixture.label_slug
        ),
    );
    write_linear_texture_meta(
        &metallic_roughness_path,
        &ambientcg_metal_texture_uri(fixture, fixture.metallic_roughness),
        &format!(
            "docs.tests.runtime.shader.ambientcg.{}.metallic_roughness",
            fixture.label_slug
        ),
    );
}

pub(super) fn ambientcg_metal009_texture_uri(file_name: &str) -> String {
    ambientcg_metal_texture_uri(AMBIENTCG_METAL009, file_name)
}

pub(super) fn ambientcg_metal_texture_uri(
    fixture: AmbientCgMetalFixture,
    file_name: &str,
) -> String {
    format!("res://textures/{}/{file_name}", fixture.asset_dir)
}

fn pack_metallic_roughness_texture(roughness_path: &Path, metalness_path: &Path, output: &Path) {
    let roughness = image::open(roughness_path)
        .unwrap_or_else(|error| panic!("read roughness texture {roughness_path:?}: {error}"))
        .to_luma8();
    let metalness = image::open(metalness_path)
        .unwrap_or_else(|error| panic!("read metalness texture {metalness_path:?}: {error}"))
        .to_luma8();
    assert_eq!(
        roughness.dimensions(),
        metalness.dimensions(),
        "ambientCG roughness and metalness maps must share dimensions"
    );

    // Zircon's standard PBR shader follows the glTF-style packed convention:
    // roughness in G and metallic in B.
    let packed =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(roughness.width(), roughness.height(), |x, y| {
            Rgba([
                255,
                roughness.get_pixel(x, y).0[0],
                metalness.get_pixel(x, y).0[0],
                255,
            ])
        });
    packed
        .save_with_format(output, ImageFormat::Png)
        .expect("write packed metallic-roughness texture");
}

fn write_linear_texture_meta(path: &Path, uri: &str, label: &str) {
    let mut meta = AssetMetaDocument::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(uri).unwrap(),
        AssetKind::Texture,
    );
    meta.import_settings.insert(
        "texture_format".to_string(),
        toml::Value::String(RGBA8_UNORM_FORMAT.to_string()),
    );
    meta.import_settings
        .insert("is_srgb".to_string(), toml::Value::Boolean(false));
    meta.save(test_meta_path_for_source(path)).unwrap();
}

fn test_meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    path.with_file_name(format!("{file_name}.zmeta"))
}
