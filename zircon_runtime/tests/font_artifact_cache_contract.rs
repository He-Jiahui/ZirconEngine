use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::artifact::ArtifactStore;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::asset::{
    AssetId, AssetKind, AssetUri, FontAsset, FontAssetCmapCoverage, FontAssetCodepointRange,
    FontAssetFaceMetrics, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetLineMetrics,
    FontAssetMetadata, FontAssetParsedFace, FontAssetRenderStrategy, FontAssetSourceFormat,
    FontAssetVariableInstance, FontAssetVariationAxis, FontAssetVariationCoord, ImportedAsset,
};
use zircon_runtime::core::resource::ResourceRecord;
use zircon_runtime::text::{
    CompositeFontDescriptor, FontCultureTag, FontFamilyName, FontScript, SubFontRange,
};

#[test]
fn font_artifact_cache_roundtrips_fields_omitted_by_authoring_formats() {
    let root = unique_temp_root("default_authoring_fields");
    let paths = ProjectPaths::from_root(&root).expect("temporary project path should resolve");
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .expect("temporary project layout should be created");

    let font = FontAsset {
        source: "fonts/zircon-variable.ttf".to_string(),
        family: Some("Zircon Sans".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: vec![FontAssetFamilyMember {
            family: "Zircon Sans".to_string(),
            face_index: 0,
            weight: None,
            width_class: Some(5),
            style: Some(FontAssetFaceStyle::Normal),
            variations: Vec::new(),
        }],
        variable_instances: vec![FontAssetVariableInstance {
            name: None,
            post_script_name: Some("ZirconSans-Regular".to_string()),
            coordinates: vec![FontAssetVariationCoord {
                tag: "wght".to_string(),
                value: 400.0,
            }],
        }],
        fallback_families: Vec::new(),
        composite_font: Some(CompositeFontDescriptor {
            default_family: FontFamilyName("Zircon Sans".to_string()),
            sub_fonts: vec![SubFontRange {
                family: FontFamilyName("Zircon Han".to_string()),
                scripts: vec![FontScript::Han],
                ranges: vec![(0x4e00, 0x9fff)],
                cultures: vec![FontCultureTag::new("zh-Hans")],
            }],
        }),
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: Some(FontAssetMetadata {
            source_format: FontAssetSourceFormat::Sfnt,
            face_count: 1,
            faces: vec![FontAssetParsedFace {
                face_index: 0,
                family: Some("Zircon Sans".to_string()),
                subfamily: None,
                full_name: Some("Zircon Sans Regular".to_string()),
                post_script_name: None,
                weight: 400,
                width_class: 5,
                style: FontAssetFaceStyle::Normal,
                metrics: FontAssetFaceMetrics {
                    units_per_em: 1_000,
                    ascender: 800,
                    descender: -200,
                    line_gap: 0,
                    uses_typographic_metrics: true,
                    windows_ascender: 900,
                    windows_descender: 250,
                    underline: None,
                    strikeout: Some(FontAssetLineMetrics {
                        position: 300,
                        thickness: 50,
                    }),
                },
                variation_axes: vec![FontAssetVariationAxis {
                    tag: "wght".to_string(),
                    min: 100.0,
                    default: 400.0,
                    max: 900.0,
                    name: None,
                    hidden: false,
                }],
                named_instances: Vec::new(),
                cmap: FontAssetCmapCoverage {
                    codepoint_count: 95,
                    ranges: vec![FontAssetCodepointRange {
                        start: 0x20,
                        end: 0x7e,
                    }],
                },
            }],
        }),
    };
    let record = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Font,
        AssetUri::parse("res://fonts/zircon-sans.font.toml")
            .expect("font resource URI should parse"),
    );
    let store = ArtifactStore;

    let artifact_uri = store
        .write(&paths, &record, &ImportedAsset::Font(font.clone()))
        .expect("font artifact cache should write");
    let payload = fs::read(paths.asset_artifact_root().join(artifact_uri.path()))
        .expect("font artifact cache payload should exist");
    let loaded = store
        .read(&paths, &artifact_uri)
        .expect("font artifact cache should deserialize");

    assert!(payload.starts_with(b"ZRARTZ01"));
    assert_eq!(loaded, ImportedAsset::Font(font));

    let _ = fs::remove_dir_all(root);
}

fn unique_temp_root(name: &str) -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_font_artifact_cache_{name}_{}_{}",
        std::process::id(),
        timestamp
    ))
}
