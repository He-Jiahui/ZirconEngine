use fdsm::{
    bezier::scanline::FillRule,
    correct_error::{correct_error_msdf, correct_error_mtsdf, ErrorCorrectionConfig},
    generate::{generate_msdf, generate_mtsdf, generate_sdf},
    render::{correct_sign_msdf, correct_sign_mtsdf, correct_sign_sdf},
    shape::Shape,
    transform::Transform,
};
use image::{GrayImage, ImageBuffer, Rgb, Rgb32FImage, Rgba};
use nalgebra::{Affine2, Similarity2, Vector2};
use ttf_parser::{Face, GlyphId, Rect, Tag};

use crate::core::math::UVec2;
use crate::text::VariationCoords;

use super::geometry_preprocess::validate_outline_shape;
use super::{SdfBakeParams, SdfGlyphData, SdfGlyphGenerationError, SdfMode};

const EDGE_COLORING_SIN_ALPHA: f64 = 0.052_335_956_242_943_835;
const EDGE_COLORING_SEED_SALT: u64 = 0x7a69_7263_6f6e_6d73;

pub(crate) fn generate_distance_field_glyph(
    font_bytes: &[u8],
    face_index: u32,
    glyph_id: u16,
    params: SdfBakeParams,
) -> Result<SdfGlyphData, SdfGlyphGenerationError> {
    generate_distance_field_glyph_with_variations(
        font_bytes,
        face_index,
        glyph_id,
        params,
        &VariationCoords::default(),
    )
}

pub(crate) fn generate_distance_field_glyph_with_variations(
    font_bytes: &[u8],
    face_index: u32,
    glyph_id: u16,
    params: SdfBakeParams,
    variations: &VariationCoords,
) -> Result<SdfGlyphData, SdfGlyphGenerationError> {
    let face = parse_distance_field_face(font_bytes, face_index, variations)?;
    generate_distance_field_glyph_from_face(&face, face_index, glyph_id, params)
}

pub(crate) fn parse_distance_field_face<'a>(
    font_bytes: &'a [u8],
    face_index: u32,
    variations: &VariationCoords,
) -> Result<Face<'a>, SdfGlyphGenerationError> {
    let mut face = Face::parse(font_bytes, face_index)
        .map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(face_index))?;
    for (tag, value) in &variations.0 {
        let _ = face.set_variation(Tag::from_bytes(&tag.to_be_bytes()), *value);
    }
    Ok(face)
}

pub(crate) fn generate_distance_field_glyph_from_face(
    face: &Face<'_>,
    source_face_index: u32,
    glyph_id: u16,
    params: SdfBakeParams,
) -> Result<SdfGlyphData, SdfGlyphGenerationError> {
    let params = params.normalized();
    let glyph_id = GlyphId(glyph_id);
    let mut shape = fdsm_ttf_parser::load_shape_from_face(face, glyph_id)
        .ok_or(SdfGlyphGenerationError::MissingGlyphOutline(glyph_id.0))?;
    let bounds = face
        .glyph_bounding_box(glyph_id)
        .ok_or(SdfGlyphGenerationError::EmptyGlyphBounds(glyph_id.0))?;
    validate_outline_shape(&shape, glyph_id.0)?;

    let layout = glyph_bake_layout(&face, glyph_id, bounds, params)?;
    shape.transform(&layout.transform);
    let pixels = generate_pixels(
        shape,
        layout.size,
        params,
        edge_coloring_seed(source_face_index, glyph_id.0),
    );
    let glyph = SdfGlyphData {
        size: layout.size,
        bitmap_left: bounds.x_min as f32 * layout.scale as f32 - params.spread_px_f32(),
        bitmap_bottom: bounds.y_min as f32 * layout.scale as f32 - params.spread_px_f32(),
        advance: face.glyph_hor_advance(glyph_id).unwrap_or_default() as f32 * layout.scale as f32,
        ascent: face.ascender() as f32 * layout.scale as f32,
        pixels,
        channels: params.mode.channel_count(),
        spread_px: params.spread_px_f32(),
        mode: params.mode,
    };
    glyph.validate()?;
    Ok(glyph)
}

struct GlyphBakeLayout {
    size: UVec2,
    scale: f64,
    transform: Affine2<f64>,
}

fn glyph_bake_layout(
    face: &Face<'_>,
    glyph_id: GlyphId,
    bounds: Rect,
    params: SdfBakeParams,
) -> Result<GlyphBakeLayout, SdfGlyphGenerationError> {
    let scale = params.bake_em_px as f64 / face.units_per_em().max(1) as f64;
    let spread = params.spread_px_f32() as f64;
    let width = scaled_dimension(bounds.x_max - bounds.x_min, scale, spread)
        .ok_or(SdfGlyphGenerationError::InvalidDimensions(UVec2::ZERO))?;
    let height = scaled_dimension(bounds.y_max - bounds.y_min, scale, spread)
        .ok_or(SdfGlyphGenerationError::InvalidDimensions(UVec2::ZERO))?;
    let size = UVec2::new(width, height);
    if size.x == 0 || size.y == 0 {
        return Err(SdfGlyphGenerationError::EmptyGlyphBounds(glyph_id.0));
    }
    let translation = Vector2::new(
        spread - bounds.x_min as f64 * scale,
        spread - bounds.y_min as f64 * scale,
    );
    let transform = nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(translation, 0.0, scale));
    Ok(GlyphBakeLayout {
        size,
        scale,
        transform,
    })
}

fn scaled_dimension(extent: i16, scale: f64, spread: f64) -> Option<u32> {
    let dimension = (extent as f64 * scale + spread * 2.0).ceil();
    dimension
        .is_finite()
        .then_some(dimension)
        .filter(|dimension| *dimension > 0.0 && *dimension <= u32::MAX as f64)
        .map(|dimension| dimension as u32)
}

fn generate_pixels(
    shape: fdsm::shape::Shape<fdsm::shape::Contour>,
    size: UVec2,
    params: SdfBakeParams,
    seed: u64,
) -> Vec<u8> {
    let range = params.spread_px_f32() as f64;
    match params.mode {
        SdfMode::Sdf => generate_sdf_pixels(shape, size, range),
        SdfMode::Msdf => generate_msdf_pixels(shape, size, range, seed),
        SdfMode::Mtsdf => generate_mtsdf_pixels(shape, size, range, seed),
    }
}

fn generate_sdf_pixels(
    shape: fdsm::shape::Shape<fdsm::shape::Contour>,
    size: UVec2,
    range: f64,
) -> Vec<u8> {
    let prepared = shape.prepare();
    let mut image = GrayImage::new(size.x, size.y);
    generate_sdf(&prepared, range, &mut image);
    correct_sign_sdf(&mut image, &prepared, FillRule::Nonzero);
    image::imageops::flip_vertical_in_place(&mut image);
    image.into_raw()
}

fn generate_msdf_pixels(
    shape: fdsm::shape::Shape<fdsm::shape::Contour>,
    size: UVec2,
    range: f64,
    seed: u64,
) -> Vec<u8> {
    let colored = Shape::edge_coloring_simple(shape, EDGE_COLORING_SIN_ALPHA, seed);
    let prepared = colored.prepare();
    let mut image = Rgb32FImage::new(size.x, size.y);
    generate_msdf(&prepared, range, &mut image);
    correct_error_msdf(
        &mut image,
        &colored,
        &prepared,
        range,
        &ErrorCorrectionConfig::default(),
    );
    correct_sign_msdf(&mut image, &prepared, FillRule::Nonzero);
    image::imageops::flip_vertical_in_place(&mut image);
    pack_msdf_rgba(image)
}

fn generate_mtsdf_pixels(
    shape: fdsm::shape::Shape<fdsm::shape::Contour>,
    size: UVec2,
    range: f64,
    seed: u64,
) -> Vec<u8> {
    let colored = Shape::edge_coloring_simple(shape, EDGE_COLORING_SIN_ALPHA, seed);
    let prepared = colored.prepare();
    let mut image = ImageBuffer::<Rgba<f32>, Vec<f32>>::new(size.x, size.y);
    generate_mtsdf(&prepared, range, &mut image);
    correct_error_mtsdf(
        &mut image,
        &colored,
        &prepared,
        range,
        &ErrorCorrectionConfig::default(),
    );
    correct_sign_mtsdf(&mut image, &prepared, FillRule::Nonzero);
    image::imageops::flip_vertical_in_place(&mut image);
    pack_float_channels(image.into_raw())
}

fn pack_msdf_rgba(image: ImageBuffer<Rgb<f32>, Vec<f32>>) -> Vec<u8> {
    let mut output = Vec::with_capacity(image.width() as usize * image.height() as usize * 4);
    for pixel in image.pixels() {
        output.extend(pixel.0.map(float_channel_to_u8));
        output.push(u8::MAX);
    }
    output
}

fn pack_float_channels(channels: Vec<f32>) -> Vec<u8> {
    channels.into_iter().map(float_channel_to_u8).collect()
}

fn float_channel_to_u8(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * u8::MAX as f32).round() as u8
}

fn edge_coloring_seed(face_index: u32, glyph_id: u16) -> u64 {
    EDGE_COLORING_SEED_SALT ^ ((face_index as u64) << 32) ^ glyph_id as u64
}
