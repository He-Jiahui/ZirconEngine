use std::fs;

use image::{ImageBuffer, Rgba};
use zircon_runtime::asset::{
    AssetUri, SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasPadding, SpriteAtlasRect,
    SpriteAtlasUvRect,
};

use super::super::{resolve_editor_sprite_atlas_image, ATLAS_LIBRARY_DIR};
use super::support::unique_temp_root;

#[test]
fn resolver_reads_project_library_atlas_artifacts_for_template_icon() {
    let root = unique_temp_root("sprite_atlas_resolver_project_library");
    let asset_path = root.join("assets").join("icons").join("search.png");
    let atlas_dir = root.join("library").join(ATLAS_LIBRARY_DIR);
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    fs::create_dir_all(&atlas_dir).unwrap();
    fs::write(&asset_path, b"source image placeholder").unwrap();

    let atlas = SpriteAtlasAsset {
        atlas_texture: AssetUri::parse("lib://editor-sprite-atlases/icons.png").unwrap(),
        width: 2,
        height: 1,
        padding: SpriteAtlasPadding::default(),
        entries: vec![SpriteAtlasEntry {
            name: "search".to_string(),
            source: Some(AssetUri::parse("res://icons/search.png").unwrap()),
            pixel_rect: SpriteAtlasRect {
                x: 1,
                y: 0,
                width: 1,
                height: 1,
            },
            uv_rect: SpriteAtlasUvRect {
                min: [0.5, 0.0],
                max: [1.0, 1.0],
            },
            source_width: 1,
            source_height: 1,
        }],
    };
    fs::write(
        atlas_dir.join("icons.toml"),
        toml::to_string_pretty(&atlas).unwrap(),
    )
    .unwrap();
    let image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255])
            .unwrap();
    image.save(atlas_dir.join("icons.png")).unwrap();

    let resolved = resolve_editor_sprite_atlas_image("template-icon:search", &asset_path)
        .expect("template icon should resolve through project library atlas artifacts");

    assert_eq!(
        resolved.resource_key,
        "lib://editor-sprite-atlases/icons.png"
    );
    assert_eq!((resolved.width, resolved.height), (2, 1));
    assert_eq!(resolved.uv.min, [0.5, 0.0]);
    assert_eq!(resolved.uv.max, [1.0, 1.0]);
    assert_eq!(
        resolved.rgba.as_deref(),
        Some(&[255, 0, 0, 255, 0, 0, 255, 255][..])
    );

    let _ = fs::remove_dir_all(root);
}
