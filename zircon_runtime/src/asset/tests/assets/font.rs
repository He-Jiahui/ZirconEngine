use std::fs;
use std::path::PathBuf;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImportError, AssetImporter, AssetKind, AssetUri, FontAsset, FontAssetError,
    FontAssetRenderStrategy, FontAssetSourceFormat, ImportedAsset,
};
use crate::text::{FontScript, FontScriptTag};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

const FONT_TOML: &str = r#"
source = "FiraMono-subset.ttf"
family = "Fira Mono"
render_mode = "sdf"
"#;

const COMPOSITE_FONT_TOML: &str = r#"
source = "FiraMono-subset.ttf"
family = "Fira Mono"

[composite_font]
default_family = "Fira Mono"

[[composite_font.sub_fonts]]
family = "Noto Sans CJK SC"
scripts = ["han"]
ranges = [[13312, 19903], [19968, 40959]]
cultures = ["zh-Hans"]
"#;

#[test]
fn standalone_collection_face_copies_each_table_directly_into_the_output() {
    let source = include_str!("../../assets/font_source.rs");
    assert!(!source.contains("let mut table_data = source_data.to_vec();"));
    assert!(source.contains("output.extend_from_slice(source_data);"));
}

#[test]
fn font_importer_version_invalidates_pre_cooked_blob_artifacts() {
    let importer = AssetImporter::default();
    let descriptor = importer
        .registry()
        .descriptor_for_source(std::path::Path::new("fonts/default.font.toml"))
        .expect("font manifest importer should be registered");

    assert_eq!(descriptor.id, "zircon.builtin.toml.font");
    assert_eq!(descriptor.importer_version, 2);
}

#[test]
fn font_asset_wrapper_parses_runtime_font_manifest_fields() {
    let font = FontAsset::from_toml_str(FONT_TOML).unwrap();

    assert_eq!(font.source, "FiraMono-subset.ttf");
    assert_eq!(font.family.as_deref(), Some("Fira Mono"));
    assert_eq!(font.render_mode, Some(UiTextRenderMode::Sdf));
    assert_eq!(font.effective_render_mode(), Some(UiTextRenderMode::Sdf));
    assert_eq!(font.face_index, 0);
    assert!(font.metadata.is_none());
}

#[test]
fn font_asset_parses_composite_font_culture_ranges() {
    let font = FontAsset::from_toml_str(COMPOSITE_FONT_TOML).unwrap();
    let composite = font
        .composite_font
        .as_ref()
        .expect("composite font descriptor should be preserved");

    assert_eq!(composite.default_family.as_str(), "Fira Mono");
    assert_eq!(composite.sub_fonts.len(), 1);
    assert_eq!(composite.sub_fonts[0].family.as_str(), "Noto Sans CJK SC");
    assert_eq!(composite.sub_fonts[0].cultures[0].as_str(), "zh-Hans");
}

#[test]
fn font_script_tag_preserves_the_existing_packed_numeric_serde_shape() {
    let packed = u32::from_be_bytes(*b"Cher");
    let tag = FontScriptTag::parse("Cher").expect("canonical ISO 15924 tag");

    assert_eq!(tag.packed(), packed);
    assert_eq!(
        serde_json::to_value(tag).unwrap(),
        serde_json::json!(packed)
    );

    let script = FontScript::Other(tag);
    let serialized = serde_json::to_value(script).unwrap();
    assert_eq!(serialized, serde_json::json!({ "other": packed }));
    assert_eq!(
        serde_json::from_value::<FontScript>(serialized).unwrap(),
        script
    );
}

#[test]
fn font_script_tag_rejects_non_canonical_and_non_alpha_packed_values() {
    assert!(FontScriptTag::parse("cher").is_none());
    assert!(FontScriptTag::parse("Ch3r").is_none());
    assert!(FontScriptTag::parse("Cherokee").is_none());

    let malformed = serde_json::json!({
        "other": u32::from_be_bytes(*b"cher")
    });
    let error = serde_json::from_value::<FontScript>(malformed)
        .expect_err("non-canonical packed values must not enter the asset schema");
    assert!(error.to_string().contains("ISO 15924"));
}

#[test]
fn runtime_default_font_manifest_declares_culture_aware_composite_font() {
    let document = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/default.font.toml"),
    )
    .unwrap();
    let font = FontAsset::from_toml_str(&document).unwrap();
    assert_eq!(font.source, "ZirconDefaultComposite-subset.ttc");
    assert_eq!(font.family.as_deref(), Some("Fira Mono"));
    assert!(font
        .family_members
        .iter()
        .any(|member| { member.family == "Fira Mono" && member.face_index == 0 }));
    assert!(font.family_members.iter().any(|member| {
        member.family == "Zircon Noto Sans CJK SC Proof" && member.face_index == 1
    }));

    let composite = font
        .composite_font
        .expect("runtime default font should declare a CompositeFont");

    assert_eq!(composite.default_family.as_str(), "Fira Mono");
    assert!(composite.sub_fonts.iter().any(|sub_font| {
        sub_font.family.as_str() == "Zircon Noto Sans CJK SC Proof"
            && sub_font
                .cultures
                .iter()
                .any(|culture| culture.as_str() == "zh-Hans")
    }));
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/fonts/")
        .join(&font.source);
    let bytes = fs::read(source_path).expect("checked-in default CompositeFont source");
    assert!(ttf_parser::Face::parse(&bytes, 0).is_ok());
    assert!(ttf_parser::Face::parse(&bytes, 1).is_ok());
    assert!(composite.sub_fonts.iter().any(|sub_font| {
        sub_font.family.as_str() == "Noto Sans CJK JP"
            && sub_font
                .cultures
                .iter()
                .any(|culture| culture.as_str() == "ja")
    }));
    assert!(composite.sub_fonts.iter().any(|sub_font| {
        sub_font.family.as_str() == "Noto Sans Arabic"
            && sub_font.scripts.contains(&FontScript::Arabic)
    }));
    assert!(composite.sub_fonts.iter().any(|sub_font| {
        sub_font.family.as_str() == "Noto Color Emoji"
            && sub_font
                .ranges
                .iter()
                .any(|(start, end)| *start <= 0x1F600 && 0x1F600 <= *end)
    }));
    assert!(font
        .fallback_families
        .iter()
        .any(|family| family == "Segoe UI Emoji"));
}

#[test]
fn font_asset_effective_render_mode_uses_strategy_default_and_constraints() {
    let font = FontAsset {
        source: "FiraMono-subset.ttf".to_string(),
        family: Some("Fira Mono".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        composite_font: None,
        render_strategy: FontAssetRenderStrategy {
            default_mode: Some(UiTextRenderMode::Auto),
            allow_native: Some(false),
            allow_sdf: Some(true),
        },
        metadata: None,
    };

    assert_eq!(font.effective_render_mode(), Some(UiTextRenderMode::Sdf));
}

#[test]
fn font_asset_parse_reports_typed_toml_error_source() {
    let error =
        FontAsset::from_toml_str("render_mode = 7").expect_err("invalid font TOML should fail");

    assert!(matches!(error, FontAssetError::Parse(_)));
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn importer_decodes_font_assets_from_font_toml() {
    let root = unique_temp_project_root("font_asset_import");
    fs::create_dir_all(&root).unwrap();
    let font_path = root.join("default.font.toml");
    fs::write(&font_path, FONT_TOML).unwrap();
    fs::copy(runtime_font_fixture(), root.join("FiraMono-subset.ttf")).unwrap();

    let importer = AssetImporter::default();
    let imported = importer
        .import_from_source(
            &font_path,
            &AssetUri::parse("res://fonts/default.font.toml").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Font(asset) => {
            assert_eq!(asset.source, "FiraMono-subset.ttf");
            assert_eq!(asset.render_mode, Some(UiTextRenderMode::Sdf));
            let metadata = asset
                .metadata
                .as_ref()
                .expect("font import should parse source metadata");
            assert_eq!(metadata.face_count, 1);
            assert!(metadata.faces[0].cmap.contains_codepoint('A' as u32));
            assert!(!asset.family_members.is_empty());
            let blob = metadata
                .cooked_blob
                .as_ref()
                .expect("font import must retain decoded bytes in its cooked artifact");
            assert_eq!(blob.source_format(), FontAssetSourceFormat::Sfnt);
            assert_eq!(
                blob.bytes(),
                fs::read(root.join("FiraMono-subset.ttf")).unwrap()
            );
            assert_eq!(blob.content_hash(), *blake3::hash(blob.bytes()).as_bytes());
            assert!(blob.has_valid_content_hash());
        }
        other => panic!("unexpected font import: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_woff2_decode_error_source() {
    let root = unique_temp_project_root("font_asset_invalid_woff2");
    fs::create_dir_all(&root).unwrap();
    let font_path = root.join("invalid.font.toml");
    fs::write(
        &font_path,
        "source = \"invalid.woff2\"\nfamily = \"Invalid Font\"\n",
    )
    .unwrap();
    fs::write(root.join("invalid.woff2"), b"wOF2invalid").unwrap();

    let error = AssetImporter::default()
        .import_from_source(
            &font_path,
            &AssetUri::parse("res://fonts/invalid.font.toml").unwrap(),
        )
        .expect_err("malformed WOFF2 should preserve the decoder failure");

    assert!(matches!(error, AssetImportError::FontSourceDecode { .. }));
    let decoder_error = std::error::Error::source(&error)
        .expect("import error should expose the font decoder source");
    assert!(decoder_error.source().is_some());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scans_font_assets_and_assigns_font_asset_kind() {
    let root = unique_temp_project_root("font_asset_project");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "FontSandbox",
        AssetUri::parse("res://fonts/default.font.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let font_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("fonts");
    fs::create_dir_all(&font_dir).unwrap();
    fs::write(font_dir.join("default.font.toml"), FONT_TOML).unwrap();
    fs::copy(runtime_font_fixture(), font_dir.join("FiraMono-subset.ttf")).unwrap();
    let expected_cooked_bytes = fs::read(font_dir.join("FiraMono-subset.ttf")).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(imported.len(), 1);
    let record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://fonts/default.font.toml").unwrap())
        .unwrap();
    assert_eq!(record.kind, AssetKind::Font);

    // The artifact is the packaged runtime input. Once import has completed, reopening the
    // project source file must not be necessary to acquire font bytes for shaping or raster.
    fs::remove_file(font_dir.join("FiraMono-subset.ttf")).unwrap();

    match manager
        .load_artifact(&AssetUri::parse("res://fonts/default.font.toml").unwrap())
        .unwrap()
    {
        ImportedAsset::Font(asset) => {
            assert_eq!(asset.family.as_deref(), Some("Fira Mono"));
            let metadata = asset
                .metadata
                .expect("font artifact should retain parsed metadata");
            let blob = metadata
                .cooked_blob
                .expect("font artifact should retain cooked decoded bytes");
            assert_eq!(blob.bytes(), expected_cooked_bytes);
            assert_eq!(blob.content_hash(), *blake3::hash(blob.bytes()).as_bytes());
            assert!(blob.has_valid_content_hash());
        }
        other => panic!("unexpected project font asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

fn runtime_font_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf")
}
