//! CPU texture loading and built-in patterns.

use image::{DynamicImage, GenericImageView};
use thiserror::Error;

use crate::asset::types::{CpuTexturePayload, TextureSource};

const RGBA8_BYTES_PER_PIXEL: usize = 4;
const CHECKER_TEXTURE_WIDTH: usize = 128;
const CHECKER_TEXTURE_HEIGHT: usize = 128;
const CHECKER_TILE_SIZE: usize = 16;
const GRID_TEXTURE_WIDTH: usize = 256;
const GRID_TEXTURE_HEIGHT: usize = 256;
const GRID_MINOR_SPACING: usize = 16;
const GRID_MAJOR_SPACING: usize = 64;

pub(crate) type TextureLoadResult<T> = std::result::Result<T, TextureLoadError>;

#[derive(Debug, Error)]
pub(crate) enum TextureLoadError {
    #[error("open image {path}: {source}")]
    OpenImage {
        path: String,
        #[source]
        source: image::ImageError,
    },
}

pub(crate) fn load_texture(source: &TextureSource) -> TextureLoadResult<CpuTexturePayload> {
    match source {
        TextureSource::BuiltinChecker => Ok(generate_checker_texture()),
        TextureSource::BuiltinGrid => Ok(generate_grid_texture()),
        TextureSource::Path(path) => decode_image_file(path),
    }
}

pub(crate) fn decode_image_file(path: &str) -> TextureLoadResult<CpuTexturePayload> {
    let image =
        image::open(std::path::Path::new(path)).map_err(|source| TextureLoadError::OpenImage {
            path: path.to_string(),
            source,
        })?;
    Ok(image_to_payload(
        TextureSource::Path(path.to_string()),
        image,
    ))
}

fn image_to_payload(source: TextureSource, image: DynamicImage) -> CpuTexturePayload {
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();

    CpuTexturePayload {
        source,
        width,
        height,
        rgba: rgba.into_raw(),
    }
}

pub(crate) fn generate_checker_texture() -> CpuTexturePayload {
    let checker_row_templates = checker_row_templates();
    let mut rgba =
        Vec::with_capacity(CHECKER_TEXTURE_WIDTH * CHECKER_TEXTURE_HEIGHT * RGBA8_BYTES_PER_PIXEL);
    for y in 0..CHECKER_TEXTURE_HEIGHT {
        rgba.extend_from_slice(&checker_row_templates[(y / CHECKER_TILE_SIZE) % 2]);
    }

    CpuTexturePayload {
        source: TextureSource::BuiltinChecker,
        width: CHECKER_TEXTURE_WIDTH as u32,
        height: CHECKER_TEXTURE_HEIGHT as u32,
        rgba,
    }
}

pub(crate) fn generate_grid_texture() -> CpuTexturePayload {
    let grid_row_templates = grid_row_templates();
    let mut rgba =
        Vec::with_capacity(GRID_TEXTURE_WIDTH * GRID_TEXTURE_HEIGHT * RGBA8_BYTES_PER_PIXEL);
    for y in 0..GRID_TEXTURE_HEIGHT {
        let row_kind = if y % GRID_MAJOR_SPACING == 0 {
            0
        } else if y % GRID_MINOR_SPACING == 0 {
            1
        } else {
            2
        };
        rgba.extend_from_slice(&grid_row_templates[row_kind]);
    }

    CpuTexturePayload {
        source: TextureSource::BuiltinGrid,
        width: GRID_TEXTURE_WIDTH as u32,
        height: GRID_TEXTURE_HEIGHT as u32,
        rgba,
    }
}

fn checker_row_templates() -> [[u8; CHECKER_TEXTURE_WIDTH * RGBA8_BYTES_PER_PIXEL]; 2] {
    const LIGHT: [u8; 4] = [220, 220, 220, 255];
    const DARK: [u8; 4] = [40, 40, 40, 255];
    let mut rows = [[0; CHECKER_TEXTURE_WIDTH * RGBA8_BYTES_PER_PIXEL]; 2];
    for (row_kind, row) in rows.iter_mut().enumerate() {
        for (x, pixel) in row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL).enumerate() {
            let color = if (x / CHECKER_TILE_SIZE + row_kind) % 2 == 0 {
                LIGHT
            } else {
                DARK
            };
            pixel.copy_from_slice(&color);
        }
    }
    rows
}

fn grid_row_templates() -> [[u8; GRID_TEXTURE_WIDTH * RGBA8_BYTES_PER_PIXEL]; 3] {
    const MAJOR: [u8; 4] = [110, 150, 255, 255];
    const MINOR: [u8; 4] = [55, 65, 85, 255];
    const BACKGROUND: [u8; 4] = [26, 30, 38, 255];
    let mut rows = [[0; GRID_TEXTURE_WIDTH * RGBA8_BYTES_PER_PIXEL]; 3];
    for (row_kind, row) in rows.iter_mut().enumerate() {
        for (x, pixel) in row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL).enumerate() {
            let color = if row_kind == 0 || x % GRID_MAJOR_SPACING == 0 {
                MAJOR
            } else if row_kind == 1 || x % GRID_MINOR_SPACING == 0 {
                MINOR
            } else {
                BACKGROUND
            };
            pixel.copy_from_slice(&color);
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{generate_checker_texture, generate_grid_texture, CpuTexturePayload};

    #[test]
    fn optimization_batch_20260830eu_runtime556_reuses_builtin_texture_row_templates() {
        let production = include_str!("texture.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("texture production source");

        assert!(production.contains("checker_row_templates"));
        assert!(production.contains("grid_row_templates"));
        assert!(!production.contains("rgba[offset..offset + 4].copy_from_slice"));
    }

    #[test]
    fn optimization_batch_20260830eu_runtime556_preserves_builtin_texture_patterns() {
        let checker = generate_checker_texture();
        assert_eq!(pixel(&checker, 0, 0), [220, 220, 220, 255]);
        assert_eq!(pixel(&checker, 15, 15), [220, 220, 220, 255]);
        assert_eq!(pixel(&checker, 16, 0), [40, 40, 40, 255]);
        assert_eq!(pixel(&checker, 16, 16), [220, 220, 220, 255]);

        let grid = generate_grid_texture();
        assert_eq!(pixel(&grid, 0, 7), [110, 150, 255, 255]);
        assert_eq!(pixel(&grid, 7, 0), [110, 150, 255, 255]);
        assert_eq!(pixel(&grid, 16, 7), [55, 65, 85, 255]);
        assert_eq!(pixel(&grid, 7, 16), [55, 65, 85, 255]);
        assert_eq!(pixel(&grid, 7, 7), [26, 30, 38, 255]);
    }

    #[test]
    #[ignore = "deterministic performance marker"]
    fn optimization_batch_20260830eu_runtime556_builtin_row_template_benchmark() {
        const SAMPLES: usize = 9;
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for _ in 0..SAMPLES {
            let started = Instant::now();
            black_box(legacy_grid_texture());
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(generate_grid_texture());
            optimized_samples.push(started.elapsed());
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        println!(
            "RUNTIME556_BUILTIN_ROW_TEMPLATE_BENCH_V1 legacy={:?} optimized={:?}",
            legacy_samples[SAMPLES / 2],
            optimized_samples[SAMPLES / 2]
        );
    }

    fn legacy_grid_texture() -> Vec<u8> {
        const WIDTH: usize = 256;
        const HEIGHT: usize = 256;
        let mut rgba = vec![0_u8; WIDTH * HEIGHT * 4];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = if x % 64 == 0 || y % 64 == 0 {
                    [110, 150, 255, 255]
                } else if x % 16 == 0 || y % 16 == 0 {
                    [55, 65, 85, 255]
                } else {
                    [26, 30, 38, 255]
                };
                let offset = (y * WIDTH + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&color);
            }
        }
        rgba
    }

    fn pixel(payload: &CpuTexturePayload, x: usize, y: usize) -> [u8; 4] {
        let offset = (y * payload.width as usize + x) * 4;
        payload.rgba[offset..offset + 4]
            .try_into()
            .expect("RGBA pixel")
    }
}
