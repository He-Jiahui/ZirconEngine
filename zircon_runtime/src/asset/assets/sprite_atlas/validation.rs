//! 图集验证是作者数据进入资源注册和 UI 采样前的拒绝边界；同时校验条目身份、像素范围和 UV 对应关系。

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
                write!(
                    f,
                    "sprite atlas dimensions must be non-zero, got {width}x{height}"
                )
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

    let mut names = HashSet::with_capacity(asset.entries.len());
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

fn validate_entry_name<'a>(
    entry: &'a SpriteAtlasEntry,
    index: usize,
    names: &mut HashSet<&'a str>,
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
    if !names.insert(name) {
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
    match uv_value_issue(min, max) {
        Some(SpriteAtlasUvValueIssue::NonFinite) => {
            return Err(SpriteAtlasValidationError::NonFiniteUv {
                name: Some(entry.name.clone()),
                min,
                max,
            });
        }
        Some(SpriteAtlasUvValueIssue::OutOfRange) => {
            return Err(SpriteAtlasValidationError::UvOutOfRange {
                name: Some(entry.name.clone()),
                min,
                max,
            });
        }
        None => {}
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpriteAtlasUvValueIssue {
    NonFinite,
    OutOfRange,
}

fn uv_value_issue(min: [f32; 2], max: [f32; 2]) -> Option<SpriteAtlasUvValueIssue> {
    let mut out_of_range = false;
    for value in min.into_iter().chain(max) {
        if !value.is_finite() {
            return Some(SpriteAtlasUvValueIssue::NonFinite);
        }
        out_of_range |= !(0.0..=1.0).contains(&value);
    }
    out_of_range.then_some(SpriteAtlasUvValueIssue::OutOfRange)
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
#[path = "validation/tests/single_pass_uv_tests.rs"]
mod single_pass_uv_tests;

#[cfg(test)]
#[path = "tests/validation.rs"]
mod tests;
