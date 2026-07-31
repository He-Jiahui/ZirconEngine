use std::fs;

use crate::asset::{AssetKind, AssetMetaDocument, AssetMetaError, AssetUri, AssetUuid};

use super::super::unique_temp_project_root;

#[test]
fn asset_meta_v7_roundtrip_persists_source_digest_only() {
    let root = unique_temp_project_root("zmeta_v7_source_digest");
    let path = root.join("hero.png.zmeta");
    let mut meta = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://textures/hero.png").unwrap(),
        AssetKind::Texture,
    );
    meta.source_digest = "blake3:0123456789abcdef".to_string();

    meta.save(&path).unwrap();
    let encoded = fs::read_to_string(&path).unwrap();
    let decoded = AssetMetaDocument::load(&path).unwrap();

    assert!(encoded.contains("format_version = 7"));
    assert!(encoded.contains("source_digest = \"blake3:0123456789abcdef\""));
    assert!(!encoded.contains("source_hash"));
    assert_eq!(decoded.source_digest, meta.source_digest);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn asset_meta_rejects_retired_source_hash_key() {
    let document = r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://textures/hero.png"
asset_kind = "Texture"
source_hash = "retired"
"#;

    assert_eq!(
        AssetMetaDocument::from_toml_str(document).unwrap_err(),
        AssetMetaError::RetiredSourceHashField
    );
}

#[test]
fn asset_meta_rejects_v6_sidecars_instead_of_silently_upgrading() {
    let document = r#"format_version = 6
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://textures/hero.png"
asset_kind = "Texture"
source_digest = "blake3:old-schema"
"#;

    assert_eq!(
        AssetMetaDocument::from_toml_str(document).unwrap_err(),
        AssetMetaError::UnsupportedOldFormatVersion {
            found: 6,
            minimum: 7,
        }
    );
}

#[test]
fn asset_meta_classifies_format_version_shape_before_current_schema_fields() {
    let cases = [
        (
            "uuid = \"11111111-2222-4333-8444-555555555555\"\n",
            AssetMetaError::MissingFormatVersion,
        ),
        (
            "format_version = \"7\"\n",
            AssetMetaError::NonIntegerFormatVersion,
        ),
        (
            "format_version = -1\n",
            AssetMetaError::NegativeFormatVersion { found: -1 },
        ),
        (
            "format_version = 4294967296\n",
            AssetMetaError::OutOfRangeFormatVersion {
                found: 4_294_967_296,
            },
        ),
    ];

    for (document, expected) in cases {
        assert_eq!(
            AssetMetaDocument::from_toml_str(document).unwrap_err(),
            expected
        );
    }
}

#[test]
fn asset_meta_future_version_classification_precedes_retired_field_checks() {
    let document = "format_version = 8\nsource_hash = \"future-owned\"\n";

    assert_eq!(
        AssetMetaDocument::from_toml_str(document).unwrap_err(),
        AssetMetaError::UnsupportedFutureFormatVersion {
            found: 8,
            supported: 7,
        }
    );
}

#[test]
fn asset_meta_entry_rejects_unknown_nested_fields() {
    let document = r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://textures/hero.png"
asset_kind = "Texture"

[[entries]]
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://textures/hero.png"
asset_kind = "Texture"
legacy_path = "textures/hero.png"
"#;

    assert!(matches!(
        AssetMetaDocument::from_toml_str(document),
        Err(AssetMetaError::DeserializeDocument { .. })
    ));
}

#[test]
fn asset_meta_rejects_malformed_root_tags_before_set_deserialization() {
    let base = r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://textures/hero.png"
asset_kind = "Texture"
"#;
    let cases = [
        ("tags = [\"hero\", \"hero\"]\n", "duplicate"),
        ("tags = [\"\"]\n", "empty"),
        ("tags = [\" hero\"]\n", "whitespace"),
        ("tags = [\"hero\\u0001\"]\n", "control"),
    ];

    for (tags, expected) in cases {
        let error = AssetMetaDocument::from_toml_str(&format!("{base}{tags}")).unwrap_err();
        match expected {
            "duplicate" => assert!(matches!(
                error,
                AssetMetaError::DuplicateTag { ref scope, .. } if scope == "root"
            )),
            "empty" => assert!(matches!(
                error,
                AssetMetaError::EmptyTag { ref scope } if scope == "root"
            )),
            "whitespace" => assert!(matches!(
                error,
                AssetMetaError::TagHasSurroundingWhitespace { ref scope, .. }
                    if scope == "root"
            )),
            "control" => assert!(matches!(
                error,
                AssetMetaError::TagContainsControlCharacter { ref scope, .. }
                    if scope == "root"
            )),
            _ => unreachable!(),
        }
    }
}

#[test]
fn asset_meta_rejects_malformed_subasset_tags_before_set_deserialization() {
    let documents = [
        r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://bundles/hero.multi"
asset_kind = "Data"

[[entries]]
uuid = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"
url = "res://bundles/hero.multi#Texture"
asset_kind = "Texture"
tags = ["ui", "ui"]
"#,
        r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://bundles/hero.multi"
asset_kind = "Data"

[[entries]]
uuid = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"
url = "res://bundles/hero.multi#Texture"
asset_kind = "Texture"
tags = [" trailing "]
"#,
    ];

    for document in documents {
        let error = AssetMetaDocument::from_toml_str(document).unwrap_err();
        assert!(matches!(
            error,
            AssetMetaError::DuplicateTag { ref scope, .. }
                | AssetMetaError::TagHasSurroundingWhitespace { ref scope, .. }
                if scope == "entries[0]"
        ));
    }
}

#[test]
fn direct_serde_entry_points_cannot_bypass_strict_tag_validation() {
    let duplicate_root = r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://textures/hero.png"
asset_kind = "Texture"
tags = ["hero", "hero"]
"#;
    let invalid_subasset = r#"format_version = 7
uuid = "11111111-2222-4333-8444-555555555555"
url = "res://bundles/hero.multi"
asset_kind = "Data"

[[entries]]
uuid = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"
url = "res://bundles/hero.multi#Texture"
asset_kind = "Texture"
tags = [" invalid "]
"#;
    let duplicate_entry = r#"uuid = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"
url = "res://bundles/hero.multi#Texture"
asset_kind = "Texture"
tags = ["ui", "ui"]
"#;

    assert!(toml::from_str::<AssetMetaDocument>(duplicate_root).is_err());
    assert!(toml::from_str::<AssetMetaDocument>(invalid_subasset).is_err());
    assert!(toml::from_str::<crate::asset::project::AssetMetaEntry>(duplicate_entry).is_err());
}

#[test]
fn asset_meta_from_toml_reuses_the_parsed_value_for_typed_decode() {
    let source = include_str!("../../../project/meta.rs");

    assert_eq!(source.matches("toml::from_str(document)").count(), 1);
    assert!(source.contains("let meta: Self = value.try_into().map_err(deserialize_error)?"));
}
