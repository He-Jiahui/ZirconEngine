use std::path::PathBuf;

use crate::asset::{FontAsset, ProjectAssetManager};

use super::*;

#[test]
fn sdf_unshaped_recovery_uses_the_requested_font_assets_composite_face() {
    let owner = "res://fonts/sdf-scoped-composite.font.toml";
    let collection = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/fonts/ZirconDefaultComposite-subset.ttc");
    let asset = FontAsset::from_toml_str(include_str!(
        "../../../../../assets/fonts/default.font.toml"
    ))
    .expect("packaged composite fixture should parse");
    let mut font_database = FontDatabase::default();
    let registered = font_database
        .replace_font_asset(owner, &asset, &collection)
        .expect("composite font asset should register");
    let mut bake = SdfFontBakeCache::new();
    let key = SdfAtlasGlyphKey {
        glyph: '界',
        glyph_id: None,
        font_id: None,
        font_instance_id: None,
        font: Some(owner.into()),
        font_family: None,
        language: Some("zh-Hans".into()),
        font_weight: FontWeight::NORMAL.0,
        bake_params: SdfBakeParams::default(),
    };

    let faces =
        bake.resolve_faces_for_key(&key, &mut font_database, &ProjectAssetManager::default());

    assert_eq!(faces.first().copied(), Some(registered.faces[1]));
    assert_ne!(registered.faces[0], registered.faces[1]);
}
