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
            .get(&index)
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
    locations: BTreeMap<usize, PackedSourceLocation>,
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
                let locations = placements
                    .packed_locations()
                    .iter()
                    .map(|(index, (_, location))| {
                        (
                            *index,
                            PackedSourceLocation {
                                x: location.x(),
                                y: location.y(),
                                width: location.width(),
                                height: location.height(),
                            },
                        )
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
mod tests {
    use zircon_runtime::asset::AssetUri;

    use super::*;

    fn source(name: &str, width: u32, height: u32, color: [u8; 4]) -> SpriteAtlasSourceImage {
        let mut rgba = Vec::new();
        for _ in 0..(width * height) {
            rgba.extend_from_slice(&color);
        }
        SpriteAtlasSourceImage {
            name: name.to_string(),
            source_uri: Some(AssetUri::parse(&format!("res://ui/{name}.png")).unwrap()),
            width,
            height,
            rgba,
        }
    }

    fn patterned_source(name: &str, width: u32, height: u32) -> SpriteAtlasSourceImage {
        let mut rgba = Vec::new();
        for y in 0..height {
            for x in 0..width {
                rgba.extend_from_slice(&[x as u8, y as u8, (x + y) as u8, 255]);
            }
        }
        SpriteAtlasSourceImage {
            name: name.to_string(),
            source_uri: Some(AssetUri::parse(&format!("res://ui/{name}.png")).unwrap()),
            width,
            height,
            rgba,
        }
    }

    #[test]
    fn sprite_atlas_packer_packs_sources_in_deterministic_entry_order() {
        let config = SpriteAtlasBuildConfig {
            initial_size: (8, 8),
            max_size: (8, 8),
            ..Default::default()
        };
        let sources = vec![
            source("search", 2, 2, [255, 0, 0, 255]),
            source("close", 2, 2, [0, 255, 0, 255]),
            source("play", 2, 2, [0, 0, 255, 255]),
        ];

        let packed = pack_sprite_atlas_sources(&config, &sources).expect("sources should pack");

        let names = packed
            .atlas
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["search", "close", "play"]);
        assert_eq!(packed.diagnostics.source_count, 3);
        assert_eq!(packed.diagnostics.packed_count, 3);
        assert_eq!(packed.diagnostics.atlas_width, 8);
        assert_eq!(packed.diagnostics.atlas_height, 8);
        assert_eq!(
            packed.atlas.atlas_texture.to_string(),
            "lib://editor-sprite-atlases/editor-atlas.png"
        );
    }

    #[test]
    fn sprite_atlas_packer_copies_source_rgba_rows_into_atlas() {
        let config = SpriteAtlasBuildConfig {
            padding: (0, 0),
            initial_size: (4, 4),
            max_size: (4, 4),
            ..Default::default()
        };
        let sources = vec![patterned_source("search", 2, 2)];

        let packed = pack_sprite_atlas_sources(&config, &sources).expect("source should pack");
        let rect = packed.atlas.entries[0].pixel_rect;
        let atlas_width = packed.atlas.width as usize;

        for y in rect.y..(rect.y + rect.height) {
            for x in rect.x..(rect.x + rect.width) {
                let offset = ((y as usize * atlas_width) + x as usize) * RGBA8_BYTES_PER_PIXEL;
                let local_x = x - rect.x;
                let local_y = y - rect.y;
                assert_eq!(
                    &packed.rgba[offset..offset + 4],
                    &[local_x as u8, local_y as u8, (local_x + local_y) as u8, 255]
                );
            }
        }
    }

    #[test]
    fn sprite_atlas_packer_derives_uvs_from_pixel_rect_without_padding() {
        let config = SpriteAtlasBuildConfig {
            padding: (1, 1),
            initial_size: (4, 4),
            max_size: (4, 4),
            ..Default::default()
        };
        let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

        let packed = pack_sprite_atlas_sources(&config, &sources).expect("source should pack");
        let entry = &packed.atlas.entries[0];

        assert_eq!(entry.pixel_rect.x, 0);
        assert_eq!(entry.pixel_rect.y, 0);
        assert_eq!(entry.pixel_rect.width, 2);
        assert_eq!(entry.pixel_rect.height, 2);
        assert_eq!(entry.uv_rect.min, [0.0, 0.0]);
        assert_eq!(entry.uv_rect.max, [0.5, 0.5]);
    }

    #[test]
    fn sprite_atlas_packer_keeps_padding_out_of_pixel_rects() {
        let config = SpriteAtlasBuildConfig {
            padding: (2, 3),
            initial_size: (8, 8),
            max_size: (8, 8),
            ..Default::default()
        };
        let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

        let packed = pack_sprite_atlas_sources(&config, &sources).expect("source should pack");

        assert_eq!(packed.atlas.padding.x, 2);
        assert_eq!(packed.atlas.padding.y, 3);
        assert_eq!(packed.atlas.entries[0].pixel_rect.width, 2);
        assert_eq!(packed.atlas.entries[0].pixel_rect.height, 2);
    }

    #[test]
    fn sprite_atlas_packer_reports_pack_failure_when_max_size_is_too_small() {
        let config = SpriteAtlasBuildConfig {
            initial_size: (2, 2),
            max_size: (2, 2),
            ..Default::default()
        };
        let sources = vec![source("search", 4, 4, [255, 0, 0, 255])];

        let error = pack_sprite_atlas_sources(&config, &sources).expect_err("pack should fail");
        let SpriteAtlasBuildError::PackFailed {
            max_width,
            max_height,
            diagnostics,
        } = error
        else {
            panic!("unexpected error: {error:?}");
        };
        assert_eq!(max_width, 2);
        assert_eq!(max_height, 2);
        assert_eq!(diagnostics.source_count, 1);
        assert_eq!(diagnostics.packed_count, 0);
        assert_eq!(diagnostics.packed_area, 25);
        assert_eq!(diagnostics.atlas_area, 4);
        assert_eq!(diagnostics.skipped_sources, vec!["search".to_string()]);
    }

    #[test]
    fn sprite_atlas_packer_rejects_unsafe_output_stem() {
        let config = SpriteAtlasBuildConfig {
            output_stem: "../icons".to_string(),
            initial_size: (4, 4),
            max_size: (4, 4),
            ..Default::default()
        };
        let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

        assert_eq!(
            pack_sprite_atlas_sources(&config, &sources),
            Err(SpriteAtlasBuildError::InvalidConfig(
                "output_stem must be a single safe file stem".to_string()
            ))
        );
    }

    #[test]
    fn sprite_atlas_packer_rejects_uri_metacharacter_output_stem() {
        let config = SpriteAtlasBuildConfig {
            output_stem: "icons#dark".to_string(),
            initial_size: (4, 4),
            max_size: (4, 4),
            ..Default::default()
        };
        let sources = vec![source("search", 2, 2, [255, 0, 0, 255])];

        assert_eq!(
            pack_sprite_atlas_sources(&config, &sources),
            Err(SpriteAtlasBuildError::InvalidConfig(
                "output_stem must be a single safe file stem".to_string()
            ))
        );
    }

    #[test]
    fn sprite_atlas_packer_keeps_padded_sources_separate() {
        let config = SpriteAtlasBuildConfig {
            padding: (1, 0),
            initial_size: (6, 2),
            max_size: (6, 2),
            ..Default::default()
        };
        let sources = vec![
            source("left", 2, 2, [255, 0, 0, 255]),
            source("right", 2, 2, [0, 255, 0, 255]),
        ];

        let packed = pack_sprite_atlas_sources(&config, &sources).expect("sources should pack");
        let first = packed.atlas.entries[0].pixel_rect;
        let second = packed.atlas.entries[1].pixel_rect;
        let separated_horizontally = first.x + first.width + config.padding.0 <= second.x
            || second.x + second.width + config.padding.0 <= first.x;
        let separated_vertically = first.y + first.height + config.padding.1 <= second.y
            || second.y + second.height + config.padding.1 <= first.y;

        assert!(separated_horizontally || separated_vertically);
    }

    #[test]
    fn sprite_atlas_packer_decodes_source_images_to_rgba8() {
        let image = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_pixel(
            2,
            1,
            image::Rgba([3, 5, 7, 255]),
        );
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();

        let source = decode_sprite_atlas_source_image(
            "decoded",
            Some(AssetUri::parse("res://ui/decoded.png").unwrap()),
            encoded.get_ref(),
        )
        .unwrap();

        assert_eq!(source.width, 2);
        assert_eq!(source.height, 1);
        assert_eq!(source.rgba, vec![3, 5, 7, 255, 3, 5, 7, 255]);
    }
}
