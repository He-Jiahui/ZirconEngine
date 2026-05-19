use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use super::layout::{SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasRect, SpriteAtlasUvRect};

const UV_DERIVATION_EPSILON: f32 = 0.000_001;

#[derive(Clone, Debug, PartialEq)]
pub enum SpriteAtlasValidationError {
    ZeroAtlasDimensions {
        width: u32,
        height: u32,
    },
    EmptyEntries,
    EmptyEntryName {
        index: usize,
    },
    EntryNameHasOuterWhitespace {
        index: usize,
        name: String,
    },
    DuplicateEntryName {
        name: String,
    },
    ZeroEntryDimensions {
        name: Option<String>,
        width: u32,
        height: u32,
    },
    ZeroSourceDimensions {
        name: Option<String>,
        source_width: u32,
        source_height: u32,
    },
    SourceDimensionsSmallerThanPixelRect {
        name: Option<String>,
        source_width: u32,
        source_height: u32,
        pixel_width: u32,
        pixel_height: u32,
    },
    PixelRectOutOfBounds {
        name: Option<String>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        atlas_width: u32,
        atlas_height: u32,
    },
    NonFiniteUv {
        name: Option<String>,
        min: [f32; 2],
        max: [f32; 2],
    },
    UvOutOfRange {
        name: Option<String>,
        min: [f32; 2],
        max: [f32; 2],
    },
    InvalidUvOrdering {
        name: Option<String>,
        min: [f32; 2],
        max: [f32; 2],
    },
    UvRectMismatch {
        name: Option<String>,
        expected_min: [f32; 2],
        expected_max: [f32; 2],
        actual_min: [f32; 2],
        actual_max: [f32; 2],
    },
}

impl Display for SpriteAtlasValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroAtlasDimensions { width, height } => {
                write!(f, "sprite atlas dimensions must be non-zero, got {width}x{height}")
            }
            Self::EmptyEntries => write!(f, "sprite atlas must contain at least one entry"),
            Self::EmptyEntryName { index } => {
                write!(f, "sprite atlas entry at index {index} has an empty name")
            }
            Self::EntryNameHasOuterWhitespace { index, name } => write!(
                f,
                "sprite atlas entry at index {index} has leading or trailing whitespace: {name:?}"
            ),
            Self::DuplicateEntryName { name } => {
                write!(f, "sprite atlas entry name is duplicated: {name}")
            }
            Self::ZeroEntryDimensions {
                name,
                width,
                height,
            } => write!(
                f,
                "sprite atlas entry {} has zero pixel dimensions {width}x{height}",
                display_entry_name(name)
            ),
            Self::ZeroSourceDimensions {
                name,
                source_width,
                source_height,
            } => write!(
                f,
                "sprite atlas entry {} has zero source dimensions {source_width}x{source_height}",
                display_entry_name(name)
            ),
            Self::SourceDimensionsSmallerThanPixelRect {
                name,
                source_width,
                source_height,
                pixel_width,
                pixel_height,
            } => write!(
                f,
                "sprite atlas entry {} source dimensions {source_width}x{source_height} are smaller than pixel rect {pixel_width}x{pixel_height}",
                display_entry_name(name)
            ),
            Self::PixelRectOutOfBounds {
                name,
                x,
                y,
                width,
                height,
                atlas_width,
                atlas_height,
            } => write!(
                f,
                "sprite atlas entry {} pixel rect {x},{y} {width}x{height} exceeds atlas {atlas_width}x{atlas_height}",
                display_entry_name(name)
            ),
            Self::NonFiniteUv { name, min, max } => write!(
                f,
                "sprite atlas entry {} has non-finite uv rect min={min:?} max={max:?}",
                display_entry_name(name)
            ),
            Self::UvOutOfRange { name, min, max } => write!(
                f,
                "sprite atlas entry {} has uv rect outside 0..1 min={min:?} max={max:?}",
                display_entry_name(name)
            ),
            Self::InvalidUvOrdering { name, min, max } => write!(
                f,
                "sprite atlas entry {} has invalid uv ordering min={min:?} max={max:?}",
                display_entry_name(name)
            ),
            Self::UvRectMismatch {
                name,
                expected_min,
                expected_max,
                actual_min,
                actual_max,
            } => write!(
                f,
                "sprite atlas entry {} uv rect min={actual_min:?} max={actual_max:?} does not match expected min={expected_min:?} max={expected_max:?}",
                display_entry_name(name)
            ),
        }
    }
}

impl Error for SpriteAtlasValidationError {}

pub fn validate_sprite_atlas_asset(
    asset: &SpriteAtlasAsset,
) -> Result<(), SpriteAtlasValidationError> {
    if asset.width == 0 || asset.height == 0 {
        return Err(SpriteAtlasValidationError::ZeroAtlasDimensions {
            width: asset.width,
            height: asset.height,
        });
    }

    let mut names = HashSet::new();
    if asset.entries.is_empty() {
        return Err(SpriteAtlasValidationError::EmptyEntries);
    }
    for (index, entry) in asset.entries.iter().enumerate() {
        validate_entry_name(entry, index, &mut names)?;
        validate_entry_dimensions(entry)?;
        validate_pixel_rect(entry, asset.width, asset.height)?;
        validate_uv_rect(entry, asset.width, asset.height)?;
    }

    Ok(())
}

fn validate_entry_name(
    entry: &SpriteAtlasEntry,
    index: usize,
    names: &mut HashSet<String>,
) -> Result<(), SpriteAtlasValidationError> {
    let name = entry.name.trim();
    if name.is_empty() {
        return Err(SpriteAtlasValidationError::EmptyEntryName { index });
    }
    if name.len() != entry.name.len() {
        return Err(SpriteAtlasValidationError::EntryNameHasOuterWhitespace {
            index,
            name: entry.name.clone(),
        });
    }
    if !names.insert(name.to_string()) {
        return Err(SpriteAtlasValidationError::DuplicateEntryName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn validate_entry_dimensions(entry: &SpriteAtlasEntry) -> Result<(), SpriteAtlasValidationError> {
    if entry.pixel_rect.width == 0 || entry.pixel_rect.height == 0 {
        return Err(SpriteAtlasValidationError::ZeroEntryDimensions {
            name: Some(entry.name.clone()),
            width: entry.pixel_rect.width,
            height: entry.pixel_rect.height,
        });
    }
    if entry.source_width == 0 || entry.source_height == 0 {
        return Err(SpriteAtlasValidationError::ZeroSourceDimensions {
            name: Some(entry.name.clone()),
            source_width: entry.source_width,
            source_height: entry.source_height,
        });
    }
    if entry.source_width < entry.pixel_rect.width || entry.source_height < entry.pixel_rect.height
    {
        return Err(
            SpriteAtlasValidationError::SourceDimensionsSmallerThanPixelRect {
                name: Some(entry.name.clone()),
                source_width: entry.source_width,
                source_height: entry.source_height,
                pixel_width: entry.pixel_rect.width,
                pixel_height: entry.pixel_rect.height,
            },
        );
    }
    Ok(())
}

fn validate_pixel_rect(
    entry: &SpriteAtlasEntry,
    atlas_width: u32,
    atlas_height: u32,
) -> Result<(), SpriteAtlasValidationError> {
    let SpriteAtlasRect {
        x,
        y,
        width,
        height,
    } = entry.pixel_rect;
    let out_of_bounds = match (x.checked_add(width), y.checked_add(height)) {
        (Some(right), Some(bottom)) => right > atlas_width || bottom > atlas_height,
        _ => true,
    };
    if out_of_bounds {
        return Err(SpriteAtlasValidationError::PixelRectOutOfBounds {
            name: Some(entry.name.clone()),
            x,
            y,
            width,
            height,
            atlas_width,
            atlas_height,
        });
    }
    Ok(())
}

fn validate_uv_rect(
    entry: &SpriteAtlasEntry,
    atlas_width: u32,
    atlas_height: u32,
) -> Result<(), SpriteAtlasValidationError> {
    let SpriteAtlasUvRect { min, max } = entry.uv_rect;
    if !min.iter().chain(max.iter()).all(|value| value.is_finite()) {
        return Err(SpriteAtlasValidationError::NonFiniteUv {
            name: Some(entry.name.clone()),
            min,
            max,
        });
    }
    if !min
        .iter()
        .chain(max.iter())
        .all(|value| (0.0..=1.0).contains(value))
    {
        return Err(SpriteAtlasValidationError::UvOutOfRange {
            name: Some(entry.name.clone()),
            min,
            max,
        });
    }
    if min[0] >= max[0] || min[1] >= max[1] {
        return Err(SpriteAtlasValidationError::InvalidUvOrdering {
            name: Some(entry.name.clone()),
            min,
            max,
        });
    }
    let expected = SpriteAtlasUvRect::from_pixel_rect(entry.pixel_rect, atlas_width, atlas_height)?;
    if !uv_rects_match(entry.uv_rect, expected) {
        return Err(SpriteAtlasValidationError::UvRectMismatch {
            name: Some(entry.name.clone()),
            expected_min: expected.min,
            expected_max: expected.max,
            actual_min: min,
            actual_max: max,
        });
    }
    Ok(())
}

fn uv_rects_match(actual: SpriteAtlasUvRect, expected: SpriteAtlasUvRect) -> bool {
    actual
        .min
        .iter()
        .chain(actual.max.iter())
        .zip(expected.min.iter().chain(expected.max.iter()))
        .all(|(actual, expected)| (*actual - *expected).abs() <= UV_DERIVATION_EPSILON)
}

fn display_entry_name(name: &Option<String>) -> &str {
    name.as_deref().unwrap_or("<unnamed>")
}

#[cfg(test)]
mod tests {
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
            uv_rect: SpriteAtlasUvRect::from_pixel_rect(pixel_rect, 128, 64)
                .expect("valid uv rect"),
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
}
