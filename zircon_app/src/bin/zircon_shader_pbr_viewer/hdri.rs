use std::error::Error;
use std::fs;
use std::path::Path;

use image::ImageFormat;
use zircon_runtime::core::framework::render::{
    build_source_cubemap_irradiance_cube, SourceCubemapEnvironment,
};

const DEFAULT_ENVIRONMENT_INTENSITY: f32 = 0.65;
const HDRI_SOURCE_REVISION: u64 = 100;

pub(crate) fn source_cubemap_environment(
    hdri_path: &Path,
    requested_face_size: u32,
) -> Result<SourceCubemapEnvironment, Box<dyn Error>> {
    let bytes = fs::read(hdri_path)?;
    let image = image::load_from_memory_with_format(&bytes, ImageFormat::Hdr)?.to_rgb32f();
    let exposure = sampled_hdri_exposure(&image);
    let natural_face_size =
        zircon_runtime::core::framework::render::source_cubemap_face_size_from_equirect_height(
            image.height(),
        );
    let face_size = requested_face_size.min(natural_face_size).max(1);
    let mip_chain = zircon_runtime::core::framework::render::build_source_cubemap_from_equirect(
        face_size,
        |u, v| expose_hdr_sample(sample_hdri_bilinear(&image, u, v), exposure),
    );
    let irradiance_cube = build_source_cubemap_irradiance_cube(&mip_chain);

    let mut environment =
        SourceCubemapEnvironment::new(mip_chain, HDRI_SOURCE_REVISION, source_hash_words(&bytes))
            .with_irradiance_cube(irradiance_cube);
    environment.intensity = DEFAULT_ENVIRONMENT_INTENSITY;
    environment.rotation_radians = 0.0;
    Ok(environment)
}

fn sample_hdri_bilinear(image: &image::Rgb32FImage, u: f32, v: f32) -> [f32; 3] {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let texel_x = u.fract() * width as f32 - 0.5;
    let texel_y = v.clamp(0.0, 1.0) * height as f32 - 0.5;
    let x0 = texel_x.floor() as i32;
    let y0 = texel_y.floor() as i32;
    let tx = texel_x - texel_x.floor();
    let ty = texel_y - texel_y.floor();
    let x0u = ((x0 % width as i32 + width as i32) % width as i32) as u32;
    let x1u = (x0u + 1) % width;
    let y0u = (y0.clamp(0, height.saturating_sub(1) as i32)) as u32;
    let y1u = (y0u + 1).min(height - 1);
    let c00 = image.get_pixel(x0u, y0u).0;
    let c10 = image.get_pixel(x1u, y0u).0;
    let c01 = image.get_pixel(x0u, y1u).0;
    let c11 = image.get_pixel(x1u, y1u).0;
    [
        lerp(lerp(c00[0], c10[0], tx), lerp(c01[0], c11[0], tx), ty),
        lerp(lerp(c00[1], c10[1], tx), lerp(c01[1], c11[1], tx), ty),
        lerp(lerp(c00[2], c10[2], tx), lerp(c01[2], c11[2], tx), ty),
    ]
}

fn sampled_hdri_exposure(image: &image::Rgb32FImage) -> f32 {
    let step_x = (image.width() / 128).max(1);
    let step_y = (image.height() / 64).max(1);
    let mut sum = 0.0_f32;
    let mut count = 0.0_f32;
    let mut y = 0;
    while y < image.height() {
        let mut x = 0;
        while x < image.width() {
            sum += luma(image.get_pixel(x, y).0);
            count += 1.0;
            x += step_x;
        }
        y += step_y;
    }
    (0.45 / (sum / count.max(1.0)).max(0.0001)).clamp(0.02, 4.0)
}

fn expose_hdr_sample(rgb: [f32; 3], exposure: f32) -> [f32; 4] {
    let exposed = rgb.map(|channel| (channel.max(0.0) * exposure).min(65_504.0));
    [exposed[0], exposed[1], exposed[2], 1.0]
}

fn source_hash_words(bytes: &[u8]) -> [u32; 4] {
    let mut state = [0x811c9dc5_u32, 0x9e3779b9, 0x85ebca6b, 0xc2b2ae35];
    for (index, byte) in bytes.iter().enumerate() {
        let slot = index & 3;
        state[slot] ^= u32::from(*byte);
        state[slot] = state[slot].wrapping_mul(16_777_619);
    }
    state
}

fn luma(rgb: [f32; 3]) -> f32 {
    rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
