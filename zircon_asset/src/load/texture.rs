//! CPU texture loading and built-in patterns.

use image::{DynamicImage, GenericImageView};

use crate::types::{CpuTexturePayload, TextureSource};

pub(crate) fn load_texture(source: &TextureSource) -> Result<CpuTexturePayload, String> {
    match source {
        TextureSource::BuiltinChecker => Ok(generate_checker_texture()),
        TextureSource::BuiltinGrid => Ok(generate_grid_texture()),
        TextureSource::Path(path) => decode_image_file(path),
    }
}

pub(crate) fn decode_image_file(path: &str) -> Result<CpuTexturePayload, String> {
    let image = image::open(std::path::Path::new(path))
        .map_err(|error| format!("open image {path}: {error}"))?;
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
    let width = 128;
    let height = 128;
    let mut rgba = vec![0_u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let tile = ((x / 16) + (y / 16)) % 2;
            let color = if tile == 0 {
                [220, 220, 220, 255]
            } else {
                [40, 40, 40, 255]
            };
            let offset = (y * width + x) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    CpuTexturePayload {
        source: TextureSource::BuiltinChecker,
        width: width as u32,
        height: height as u32,
        rgba,
    }
}

pub(crate) fn generate_grid_texture() -> CpuTexturePayload {
    let width = 256;
    let height = 256;
    let mut rgba = vec![0_u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let is_major = x % 64 == 0 || y % 64 == 0;
            let is_minor = x % 16 == 0 || y % 16 == 0;
            let color = if is_major {
                [110, 150, 255, 255]
            } else if is_minor {
                [55, 65, 85, 255]
            } else {
                [26, 30, 38, 255]
            };
            let offset = (y * width + x) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    CpuTexturePayload {
        source: TextureSource::BuiltinGrid,
        width: width as u32,
        height: height as u32,
        rgba,
    }
}
