use crate::asset::AssetUri;

use super::super::{
    SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasPadding, SpriteAtlasRect, SpriteAtlasUvRect,
};
use super::*;

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).expect("test locator should parse")
}

fn valid_entry(name: &str) -> SpriteAtlasEntry {
    let pixel_rect = SpriteAtlasRect {
        x: 8,
        y: 16,
        width: 32,
        height: 16,
    };
    SpriteAtlasEntry {
        name: name.to_string(),
        source: Some(uri("res://ui/icons/source.png")),
        pixel_rect,
        uv_rect: SpriteAtlasUvRect::from_pixel_rect(pixel_rect, 128, 64).expect("valid uv rect"),
        source_width: 32,
        source_height: 16,
    }
}

fn valid_asset() -> SpriteAtlasAsset {
    SpriteAtlasAsset {
        atlas_texture: uri("res://ui/atlas/editor.png"),
        width: 128,
        height: 64,
        padding: SpriteAtlasPadding { x: 1, y: 1 },
        entries: vec![valid_entry("search")],
    }
}

#[test]
fn sprite_atlas_validation_accepts_valid_asset() {
    let asset = valid_asset();

    validate_sprite_atlas_asset(&asset).expect("valid sprite atlas should pass validation");
}

#[test]
fn sprite_atlas_asset_roundtrips_through_toml_and_remains_valid() {
    let asset = valid_asset();

    let encoded = toml::to_string_pretty(&asset).expect("sprite atlas should serialize");
    let decoded: SpriteAtlasAsset =
        toml::from_str(&encoded).expect("sprite atlas should deserialize");

    assert_eq!(decoded, asset);
    validate_sprite_atlas_asset(&decoded).expect("decoded sprite atlas should remain valid");
}

#[test]
fn sprite_atlas_validation_rejects_zero_atlas_size() {
    let mut asset = valid_asset();
    asset.width = 0;

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::ZeroAtlasDimensions {
            width: 0,
            height: 64
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_empty_entry_name() {
    let mut asset = valid_asset();
    asset.entries[0].name = "  ".to_string();

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::EmptyEntryName { index: 0 })
    );
}

#[test]
fn sprite_atlas_validation_rejects_empty_entries() {
    let mut asset = valid_asset();
    asset.entries.clear();

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::EmptyEntries)
    );
}

#[test]
fn sprite_atlas_validation_rejects_duplicate_entry_names() {
    let mut asset = valid_asset();
    asset.entries.push(valid_entry("search"));

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::DuplicateEntryName {
            name: "search".to_string()
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_whitespace_variant_entry_names() {
    let mut asset = valid_asset();
    asset.entries.push(valid_entry(" search "));

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::EntryNameHasOuterWhitespace {
            index: 1,
            name: " search ".to_string(),
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_out_of_bounds_pixel_rect() {
    let mut asset = valid_asset();
    asset.entries[0].pixel_rect.x = 120;

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::PixelRectOutOfBounds {
            name: Some("search".to_string()),
            x: 120,
            y: 16,
            width: 32,
            height: 16,
            atlas_width: 128,
            atlas_height: 64,
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_zero_entry_and_source_dimensions() {
    let mut zero_entry = valid_asset();
    zero_entry.entries[0].pixel_rect.width = 0;
    assert_eq!(
        validate_sprite_atlas_asset(&zero_entry),
        Err(SpriteAtlasValidationError::ZeroEntryDimensions {
            name: Some("search".to_string()),
            width: 0,
            height: 16,
        })
    );

    let mut zero_source = valid_asset();
    zero_source.entries[0].source_height = 0;
    assert_eq!(
        validate_sprite_atlas_asset(&zero_source),
        Err(SpriteAtlasValidationError::ZeroSourceDimensions {
            name: Some("search".to_string()),
            source_width: 32,
            source_height: 0,
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_source_smaller_than_pixel_rect() {
    let mut asset = valid_asset();
    asset.entries[0].source_width = 16;

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(
            SpriteAtlasValidationError::SourceDimensionsSmallerThanPixelRect {
                name: Some("search".to_string()),
                source_width: 16,
                source_height: 16,
                pixel_width: 32,
                pixel_height: 16,
            }
        )
    );
}

#[test]
fn sprite_atlas_validation_rejects_non_finite_and_out_of_range_uvs() {
    let mut non_finite = valid_asset();
    non_finite.entries[0].uv_rect.min[0] = f32::NAN;
    let error =
        validate_sprite_atlas_asset(&non_finite).expect_err("non-finite uv must be rejected");
    match error {
        SpriteAtlasValidationError::NonFiniteUv { name, min, max } => {
            assert_eq!(name, Some("search".to_string()));
            assert!(min[0].is_nan());
            assert_eq!(min[1], 0.25);
            assert_eq!(max, [0.3125, 0.5]);
        }
        other => panic!("unexpected validation error: {other:?}"),
    }

    let mut out_of_range = valid_asset();
    out_of_range.entries[0].uv_rect.max[0] = 1.25;
    assert_eq!(
        validate_sprite_atlas_asset(&out_of_range),
        Err(SpriteAtlasValidationError::UvOutOfRange {
            name: Some("search".to_string()),
            min: [0.0625, 0.25],
            max: [1.25, 0.5],
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_invalid_uv_ordering() {
    let mut asset = valid_asset();
    asset.entries[0].uv_rect.max[0] = asset.entries[0].uv_rect.min[0];

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::InvalidUvOrdering {
            name: Some("search".to_string()),
            min: [0.0625, 0.25],
            max: [0.0625, 0.5],
        })
    );
}

#[test]
fn sprite_atlas_validation_rejects_uv_rect_that_does_not_match_pixel_rect() {
    let mut asset = valid_asset();
    asset.entries[0].uv_rect = SpriteAtlasUvRect {
        min: [0.5, 0.25],
        max: [0.75, 0.5],
    };

    assert_eq!(
        validate_sprite_atlas_asset(&asset),
        Err(SpriteAtlasValidationError::UvRectMismatch {
            name: Some("search".to_string()),
            expected_min: [0.0625, 0.25],
            expected_max: [0.3125, 0.5],
            actual_min: [0.5, 0.25],
            actual_max: [0.75, 0.5],
        })
    );
}
