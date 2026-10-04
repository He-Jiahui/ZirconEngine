use std::io::Cursor;
use std::path::Path;

use image::ImageDecoder;

use crate::asset::AssetImportError;

use super::super::auxiliary_source::AuxiliarySourceResolver;
use super::budget::DecodedBudget;
use super::buffers::decode_data_uri;
use super::sources::ExternalSources;
use super::{gltf_parse_error, is_data_uri, MAX_GLTF_AUXILIARY_BYTES};

const MAX_IMAGE_DECODER_ALLOCATION_BYTES: u64 = 512 * 1024 * 1024;
const RGBA_CHANNEL_COUNT: usize = 4;

pub(super) fn decode_images(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    sources: &mut ExternalSources<'_>,
    budget: &mut DecodedBudget,
) -> Result<Vec<gltf::image::Data>, AssetImportError> {
    let image_count = document.images().len();
    if image_count > AuxiliarySourceResolver::MAX_SNAPSHOT_FILES {
        return Err(gltf_parse_error(
            "gltf decoded image count exceeds its cumulative limit",
        ));
    }
    let mut images = Vec::with_capacity(image_count);
    for image in document.images() {
        let decoded = match image.source() {
            gltf::image::Source::View { view, mime_type } => {
                let buffer = buffers.get(view.buffer().index()).ok_or_else(|| {
                    gltf_parse_error(format!(
                        "gltf image {} references a missing buffer",
                        image.index()
                    ))
                })?;
                let end = view
                    .offset()
                    .checked_add(view.length())
                    .ok_or_else(|| gltf_parse_error("gltf image range overflow"))?;
                let encoded = buffer.get(view.offset()..end).ok_or_else(|| {
                    gltf_parse_error(format!(
                        "gltf image {} range is out of bounds",
                        image.index()
                    ))
                })?;
                decode_external_image(encoded, Path::new(""), Some(mime_type), budget)
            }
            gltf::image::Source::Uri { uri, mime_type } if !is_data_uri(uri) => {
                let (path, encoded) = sources.read(uri, MAX_GLTF_AUXILIARY_BYTES)?;
                decode_external_image(encoded, path, mime_type, budget)
            }
            gltf::image::Source::Uri { uri, mime_type } => {
                let encoded = decode_data_uri(uri, budget.remaining())?;
                let mime_type = mime_type.or_else(|| {
                    uri.strip_prefix("data:")
                        .and_then(|rest| rest.split_once(";base64,").map(|(mime, _)| mime))
                });
                let mut image_budget =
                    DecodedBudget::new(budget.remaining() - encoded.len() as u64);
                let image =
                    decode_external_image(&encoded, Path::new(""), mime_type, &mut image_budget)?;
                budget.charge(image.pixels.len() as u64, "image")?;
                Ok(image)
            }
        }?;
        images.push(decoded);
    }
    Ok(images)
}

pub(super) fn decode_external_image(
    encoded: &[u8],
    path: &Path,
    mime_type: Option<&str>,
    budget: &mut DecodedBudget,
) -> Result<gltf::image::Data, AssetImportError> {
    let encoding = mime_type
        .and_then(|mime| image::ImageFormat::from_mime_type(mime.to_ascii_lowercase()))
        .or_else(|| {
            path.extension()
                .and_then(image::ImageFormat::from_extension)
        })
        .or_else(|| image::guess_format(encoded).ok())
        .ok_or_else(|| gltf_parse_error("decode gltf image: unsupported image encoding"))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(budget.remaining().min(u32::MAX as u64) as u32);
    limits.max_image_height = limits.max_image_width;
    limits.max_alloc = Some(MAX_IMAGE_DECODER_ALLOCATION_BYTES);
    let mut reader = image::ImageReader::with_format(Cursor::new(encoded), encoding);
    reader.limits(limits.clone());
    let mut decoder = reader
        .into_decoder()
        .map_err(|error| gltf_parse_error(format!("decode gltf image header: {error}")))?;
    let (width, height) = decoder.dimensions();
    let color = decoder.color_type();
    let source_length = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(u64::from(color.bytes_per_pixel())))
        .ok_or_else(|| gltf_parse_error("gltf decoded image budget overflow"))?;
    if source_length != decoder.total_bytes() {
        return Err(gltf_parse_error(
            "gltf image decoder reports inconsistent byte length",
        ));
    }
    let expand_webp = encoding == image::ImageFormat::WebP && color == image::ColorType::Rgb8;
    let output_length = if expand_webp {
        u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|pixels| pixels.checked_mul(RGBA_CHANNEL_COUNT as u64))
            .ok_or_else(|| gltf_parse_error("gltf decoded image budget overflow"))?
    } else {
        source_length
    };
    let format = if expand_webp {
        gltf::image::Format::R8G8B8A8
    } else {
        gltf_image_format(color)?
    };
    // RGB WebP conversion temporarily owns both buffers; admit that peak before decoding.
    let peak = if expand_webp {
        output_length
            .checked_add(source_length)
            .ok_or_else(|| gltf_parse_error("gltf decoded image budget overflow"))?
    } else {
        output_length
    };
    budget.check(peak, "image")?;
    limits.reserve(peak).map_err(|error| {
        gltf_parse_error(format!("gltf image decoder allocation budget: {error}"))
    })?;
    decoder
        .set_limits(limits)
        .map_err(|error| gltf_parse_error(format!("set gltf image decoder limits: {error}")))?;
    let mut pixels = pixel_buffer(source_length)?;
    decoder
        .read_image(&mut pixels)
        .map_err(|error| gltf_parse_error(format!("decode gltf image: {error}")))?;
    if expand_webp {
        let mut rgba = pixel_buffer(output_length)?;
        for (rgb, rgba) in pixels
            .chunks_exact(3)
            .zip(rgba.chunks_exact_mut(RGBA_CHANNEL_COUNT))
        {
            rgba[..3].copy_from_slice(rgb);
            rgba[3] = u8::MAX;
        }
        pixels = rgba;
    }
    budget.charge(pixels.len() as u64, "image")?;
    Ok(gltf::image::Data {
        pixels,
        format,
        width,
        height,
    })
}

fn pixel_buffer(bytes: u64) -> Result<Vec<u8>, AssetImportError> {
    let length = usize::try_from(bytes)
        .map_err(|_| gltf_parse_error("gltf decoded image byte length overflow"))?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|error| gltf_parse_error(format!("allocate gltf image pixels: {error}")))?;
    output.resize(length, 0);
    Ok(output)
}

fn gltf_image_format(color: image::ColorType) -> Result<gltf::image::Format, AssetImportError> {
    Ok(match color {
        image::ColorType::L8 => gltf::image::Format::R8,
        image::ColorType::La8 => gltf::image::Format::R8G8,
        image::ColorType::Rgb8 => gltf::image::Format::R8G8B8,
        image::ColorType::Rgba8 => gltf::image::Format::R8G8B8A8,
        image::ColorType::L16 => gltf::image::Format::R16,
        image::ColorType::La16 => gltf::image::Format::R16G16,
        image::ColorType::Rgb16 => gltf::image::Format::R16G16B16,
        image::ColorType::Rgba16 => gltf::image::Format::R16G16B16A16,
        image::ColorType::Rgb32F => gltf::image::Format::R32G32B32FLOAT,
        image::ColorType::Rgba32F => gltf::image::Format::R32G32B32A32FLOAT,
        _ => {
            return Err(gltf_parse_error(
                "decode gltf image: unsupported image format",
            ))
        }
    })
}

#[cfg(test)]
#[path = "images/tests/optimization_batch_runtime868_gltf_image_output_capacity_tests.rs"]
mod optimization_batch_runtime868_gltf_image_output_capacity_tests;
