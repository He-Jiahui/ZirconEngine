use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};
use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::{
    AssetUri, SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasPadding, SpriteAtlasRect,
    SpriteAtlasUvRect,
};

use super::config::SpriteAtlasBuildConfig;
use super::diagnostics::SpriteAtlasBuildDiagnostics;

const RGBA8_BYTES_PER_PIXEL: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteAtlasSourceImage {
    pub name: String,
    pub source_uri: Option<AssetUri>,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackedSpriteAtlas {
    pub atlas: SpriteAtlasAsset,
    pub rgba: Vec<u8>,
    pub diagnostics: SpriteAtlasBuildDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpriteAtlasBuildError {
    EmptySources,
    InvalidConfig(String),
    InvalidSourceDimensions {
        name: String,
        width: u32,
        height: u32,
    },
    InvalidSourceByteLength {
        name: String,
        expected: usize,
        actual: usize,
    },
    PackFailed {
        max_width: u32,
        max_height: u32,
        diagnostics: SpriteAtlasBuildDiagnostics,
    },
    AtlasTooLarge {
        width: u32,
        height: u32,
    },
    AtlasValidation(String),
    Io(String),
    ImageDecode(String),
    Toml(String),
    Uri(String),
}

impl Display for SpriteAtlasBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySources => write!(f, "sprite atlas requires at least one source image"),
            Self::InvalidConfig(message) => write!(f, "invalid sprite atlas config: {message}"),
            Self::InvalidSourceDimensions {
                name,
                width,
                height,
            } => write!(
                f,
                "sprite atlas source {name:?} has invalid dimensions {width}x{height}"
            ),
            Self::InvalidSourceByteLength {
                name,
                expected,
                actual,
            } => write!(
                f,
                "sprite atlas source {name:?} expected {expected} RGBA bytes but found {actual}"
            ),
            Self::PackFailed {
                max_width,
                max_height,
                diagnostics,
            } => write!(
                f,
                "could not pack {} sprite atlas sources into max size {max_width}x{max_height}: {}",
                diagnostics.source_count, diagnostics.message
            ),
            Self::AtlasTooLarge { width, height } => {
                write!(f, "sprite atlas size {width}x{height} is too large")
            }
            Self::AtlasValidation(message) => {
                write!(f, "sprite atlas validation failed: {message}")
            }
            Self::Io(message) => write!(f, "sprite atlas artifact I/O failed: {message}"),
            Self::ImageDecode(message) => write!(f, "sprite atlas image decode failed: {message}"),
            Self::Toml(message) => {
                write!(f, "sprite atlas manifest serialization failed: {message}")
            }
            Self::Uri(message) => write!(f, "sprite atlas uri failed: {message}"),
        }
    }
}

impl Error for SpriteAtlasBuildError {}

pub fn pack_sprite_atlas_sources(
    config: &SpriteAtlasBuildConfig,
    sources: &[SpriteAtlasSourceImage],
) -> Result<PackedSpriteAtlas, SpriteAtlasBuildError> {
    validate_config(config)?;
    validate_sources(sources)?;

    let placements = pack_source_rects(config, sources)?;
    let atlas_len = rgba_len(placements.width, placements.height)?;
    let mut atlas_rgba = vec![0; atlas_len];
    let mut entries = Vec::with_capacity(sources.len());
    let mut packed_area = 0_u64;

    for (index, source) in sources.iter().enumerate() {
        let packed_location = placements
            .locations
            .get(index)
            .expect("rectangle-pack returns a location for each source");
        copy_source_to_atlas(source, &mut atlas_rgba, placements.width, packed_location)?;
        let pixel_rect = SpriteAtlasRect {
            x: packed_location.x,
            y: packed_location.y,
            width: source.width,
            height: source.height,
        };
        packed_area += padded_source_area(source, config.padding);
        entries.push(SpriteAtlasEntry {
            name: source.name.clone(),
            source: source.source_uri.clone(),
            pixel_rect,
            uv_rect: SpriteAtlasUvRect::from_pixel_rect(
                pixel_rect,
                placements.width,
                placements.height,
            )
            .map_err(|error| SpriteAtlasBuildError::AtlasValidation(error.to_string()))?,
            source_width: source.width,
            source_height: source.height,
        });
    }

    let atlas_texture = atlas_texture_uri(config)?;
    let atlas = SpriteAtlasAsset {
        atlas_texture,
        width: placements.width,
        height: placements.height,
        padding: SpriteAtlasPadding {
            x: config.padding.0,
            y: config.padding.1,
        },
        entries,
    };
    zircon_runtime::asset::validate_sprite_atlas_asset(&atlas)
        .map_err(|error| SpriteAtlasBuildError::AtlasValidation(error.to_string()))?;

    Ok(PackedSpriteAtlas {
        atlas,
        rgba: atlas_rgba,
        diagnostics: SpriteAtlasBuildDiagnostics {
            source_count: sources.len(),
            packed_count: sources.len(),
            atlas_width: placements.width,
            atlas_height: placements.height,
            padding: config.padding,
            packed_area,
            atlas_area: u64::from(placements.width) * u64::from(placements.height),
            skipped_sources: Vec::new(),
            message: format!(
                "packed {} sources into {}x{} sprite atlas",
                sources.len(),
                placements.width,
                placements.height
            ),
        },
    })
}

pub fn decode_sprite_atlas_source_image(
    name: impl Into<String>,
    source_uri: Option<AssetUri>,
    encoded: &[u8],
) -> Result<SpriteAtlasSourceImage, SpriteAtlasBuildError> {
    let name = name.into();
    let image = image::load_from_memory(encoded)
        .map_err(|error| SpriteAtlasBuildError::ImageDecode(error.to_string()))?
        .into_rgba8();
    let (width, height) = image.dimensions();
    Ok(SpriteAtlasSourceImage {
        name,
        source_uri,
        width,
        height,
        rgba: image.into_raw(),
    })
}

pub(super) fn atlas_texture_uri(
    config: &SpriteAtlasBuildConfig,
) -> Result<AssetUri, SpriteAtlasBuildError> {
    config
        .validate()
        .map_err(SpriteAtlasBuildError::InvalidConfig)?;
    AssetUri::parse(&format!(
        "lib://editor-sprite-atlases/{}.png",
        config.output_stem
    ))
    .map_err(|error| SpriteAtlasBuildError::Uri(error.to_string()))
}

pub(super) fn atlas_manifest_uri(
    config: &SpriteAtlasBuildConfig,
) -> Result<AssetUri, SpriteAtlasBuildError> {
    config
        .validate()
        .map_err(SpriteAtlasBuildError::InvalidConfig)?;
    AssetUri::parse(&format!(
        "lib://editor-sprite-atlases/{}.toml",
        config.output_stem
    ))
    .map_err(|error| SpriteAtlasBuildError::Uri(error.to_string()))
}

impl From<std::io::Error> for SpriteAtlasBuildError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<image::ImageError> for SpriteAtlasBuildError {
    fn from(error: image::ImageError) -> Self {
        Self::ImageDecode(error.to_string())
    }
}

impl From<toml::ser::Error> for SpriteAtlasBuildError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Toml(error.to_string())
    }
}

impl From<zircon_runtime::core::resource::ResourceLocatorError> for SpriteAtlasBuildError {
    fn from(error: zircon_runtime::core::resource::ResourceLocatorError) -> Self {
        Self::Uri(error.to_string())
    }
}

impl From<SpriteAtlasBuildError> for AssetImportError {
    fn from(error: SpriteAtlasBuildError) -> Self {
        Self::Parse(error.to_string())
    }
}

#[derive(Clone, Debug)]
struct PackedSourceRects {
    width: u32,
    height: u32,
    locations: Vec<PackedSourceLocation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackedSourceLocation {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn validate_config(config: &SpriteAtlasBuildConfig) -> Result<(), SpriteAtlasBuildError> {
    config
        .validate()
        .map_err(SpriteAtlasBuildError::InvalidConfig)
}

fn validate_sources(sources: &[SpriteAtlasSourceImage]) -> Result<(), SpriteAtlasBuildError> {
    if sources.is_empty() {
        return Err(SpriteAtlasBuildError::EmptySources);
    }
    for source in sources {
        if source.width == 0 || source.height == 0 {
            return Err(SpriteAtlasBuildError::InvalidSourceDimensions {
                name: source.name.clone(),
                width: source.width,
                height: source.height,
            });
        }
        let expected = rgba_len(source.width, source.height)?;
        if source.rgba.len() != expected {
            return Err(SpriteAtlasBuildError::InvalidSourceByteLength {
                name: source.name.clone(),
                expected,
                actual: source.rgba.len(),
            });
        }
    }
    Ok(())
}

fn pack_source_rects(
    config: &SpriteAtlasBuildConfig,
    sources: &[SpriteAtlasSourceImage],
) -> Result<PackedSourceRects, SpriteAtlasBuildError> {
    let mut rects_to_place = GroupedRectsToPlace::<usize>::new();
    for (index, source) in sources.iter().enumerate() {
        let padded_width = source.width.checked_add(config.padding.0).ok_or(
            SpriteAtlasBuildError::AtlasTooLarge {
                width: source.width,
                height: source.height,
            },
        )?;
        let padded_height = source.height.checked_add(config.padding.1).ok_or(
            SpriteAtlasBuildError::AtlasTooLarge {
                width: source.width,
                height: source.height,
            },
        )?;
        rects_to_place.push_rect(
            index,
            None,
            RectToInsert::new(padded_width, padded_height, 1),
        );
    }

    let (mut current_width, mut current_height) = config.initial_size;
    loop {
        if current_width > config.max_size.0 || current_height > config.max_size.1 {
            return Err(SpriteAtlasBuildError::PackFailed {
                max_width: config.max_size.0,
                max_height: config.max_size.1,
                diagnostics: failure_diagnostics(config, sources),
            });
        }

        let last_attempt =
            current_width == config.max_size.0 && current_height == config.max_size.1;
        let mut target_bins = BTreeMap::new();
        target_bins.insert(0, TargetBin::new(current_width, current_height, 1));
        match pack_rects(
            &rects_to_place,
            &mut target_bins,
            &volume_heuristic,
            &contains_smallest_box,
        ) {
            Ok(placements) => {
                let mut locations = vec![None; sources.len()];
                for (index, (_, location)) in placements.packed_locations() {
                    let slot = locations
                        .get_mut(*index)
                        .expect("rectangle-pack preserves the source index domain");
                    *slot = Some(PackedSourceLocation {
                        x: location.x(),
                        y: location.y(),
                        width: location.width(),
                        height: location.height(),
                    });
                }
                let locations = locations
                    .into_iter()
                    .map(|location| {
                        location.expect("rectangle-pack returns a location for each source")
                    })
                    .collect();
                return Ok(PackedSourceRects {
                    width: current_width,
                    height: current_height,
                    locations,
                });
            }
            Err(rectangle_pack::RectanglePackError::NotEnoughBinSpace) if !last_attempt => {
                current_width = current_width.saturating_mul(2).min(config.max_size.0);
                current_height = current_height.saturating_mul(2).min(config.max_size.1);
            }
            Err(rectangle_pack::RectanglePackError::NotEnoughBinSpace) => {
                return Err(SpriteAtlasBuildError::PackFailed {
                    max_width: config.max_size.0,
                    max_height: config.max_size.1,
                    diagnostics: failure_diagnostics(config, sources),
                });
            }
        }
    }
}

fn failure_diagnostics(
    config: &SpriteAtlasBuildConfig,
    sources: &[SpriteAtlasSourceImage],
) -> SpriteAtlasBuildDiagnostics {
    SpriteAtlasBuildDiagnostics {
        source_count: sources.len(),
        packed_count: 0,
        atlas_width: config.max_size.0,
        atlas_height: config.max_size.1,
        padding: config.padding,
        packed_area: sources
            .iter()
            .map(|source| padded_source_area(source, config.padding))
            .sum(),
        atlas_area: u64::from(config.max_size.0) * u64::from(config.max_size.1),
        skipped_sources: sources.iter().map(|source| source.name.clone()).collect(),
        message: "sources do not fit within configured max_size".to_string(),
    }
}

fn copy_source_to_atlas(
    source: &SpriteAtlasSourceImage,
    atlas_rgba: &mut [u8],
    atlas_width: u32,
    packed_location: &PackedSourceLocation,
) -> Result<(), SpriteAtlasBuildError> {
    let atlas_width =
        usize::try_from(atlas_width).map_err(|_| SpriteAtlasBuildError::AtlasTooLarge {
            width: atlas_width,
            height: packed_location.y + packed_location.height,
        })?;
    let source_width =
        usize::try_from(source.width).map_err(|_| SpriteAtlasBuildError::AtlasTooLarge {
            width: source.width,
            height: source.height,
        })?;
    let rect_x = packed_location.x as usize;
    let rect_y = packed_location.y as usize;
    for row in 0..(source.height as usize) {
        let dst_start = ((rect_y + row) * atlas_width + rect_x) * RGBA8_BYTES_PER_PIXEL;
        let dst_end = dst_start + source_width * RGBA8_BYTES_PER_PIXEL;
        let src_start = row * source_width * RGBA8_BYTES_PER_PIXEL;
        let src_end = src_start + source_width * RGBA8_BYTES_PER_PIXEL;
        atlas_rgba[dst_start..dst_end].copy_from_slice(&source.rgba[src_start..src_end]);
    }
    Ok(())
}

fn rgba_len(width: u32, height: u32) -> Result<usize, SpriteAtlasBuildError> {
    width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(RGBA8_BYTES_PER_PIXEL as u32))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or(SpriteAtlasBuildError::AtlasTooLarge { width, height })
}

fn padded_source_area(source: &SpriteAtlasSourceImage, padding: (u32, u32)) -> u64 {
    (u64::from(source.width) + u64::from(padding.0))
        * (u64::from(source.height) + u64::from(padding.1))
}

#[cfg(test)]
#[path = "tests/packer.rs"]
mod tests;
