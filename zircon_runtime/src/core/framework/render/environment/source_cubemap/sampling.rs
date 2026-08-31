use crate::core::framework::render::environment::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction, CubemapFace,
};
use crate::core::math::Real;

use super::{source_cubemap_face_mip_offset, source_cubemap_mip_size};

pub(super) fn sample_source_cubemap_trilinear(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    direction: [Real; 3],
    mip_level: Real,
) -> [Real; 4] {
    let max_mip = mip_count.max(1).saturating_sub(1) as Real;
    let mip_level = mip_level.clamp(0.0, max_mip);
    let mip0 = mip_level.floor() as u32;
    let mip1 = (mip0 + 1).min(mip_count.max(1).saturating_sub(1));
    let t = mip_level - mip0 as Real;
    if mip0 == mip1 || t <= Real::EPSILON {
        return sample_cubemap_linear_at_mip(texels, face_size, mip_count, direction, mip0);
    }
    if t >= 1.0 - Real::EPSILON {
        return sample_cubemap_linear_at_mip(texels, face_size, mip_count, direction, mip1);
    }
    lerp4(
        sample_cubemap_linear_at_mip(texels, face_size, mip_count, direction, mip0),
        sample_cubemap_linear_at_mip(texels, face_size, mip_count, direction, mip1),
        t,
    )
}

pub(super) fn sample_cubemap_linear_at_mip(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    direction: [Real; 3],
    mip_level: u32,
) -> [Real; 4] {
    let (face, scaled_uv) = cubemap_face_scaled_uv_from_direction(direction);
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let texel_x = (scaled_uv[0] * 0.5 + 0.5) * mip_size as Real - 0.5;
    let texel_y = (scaled_uv[1] * 0.5 + 0.5) * mip_size as Real - 0.5;
    let x0 = texel_x.floor();
    let y0 = texel_y.floor();
    let tx = texel_x - x0;
    let ty = texel_y - y0;
    let x0 = x0 as i32;
    let y0 = y0 as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let c00 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x0, y0);
    let c10 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x1, y0);
    let c01 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x0, y1);
    let c11 = sample_cubemap_texel_unwrapped(texels, face_size, mip_count, face, mip_level, x1, y1);
    lerp4(lerp4(c00, c10, tx), lerp4(c01, c11, tx), ty)
}

fn sample_cubemap_texel_unwrapped(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
    x: i32,
    y: i32,
) -> [Real; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let mip_size_i32 = mip_size as i32;
    if x >= 0 && x < mip_size_i32 && y >= 0 && y < mip_size_i32 {
        return mip_texel(
            texels, face_size, mip_count, face, mip_level, x as u32, y as u32,
        );
    }

    // Match cmft-style neighbour bleed by projecting out-of-face taps back through cube space.
    let scaled_uv = [
        ((x as Real + 0.5) / mip_size as Real) * 2.0 - 1.0,
        ((y as Real + 0.5) / mip_size as Real) * 2.0 - 1.0,
    ];
    let direction = cubemap_direction_from_scaled_uv(face, scaled_uv);
    let (sample_face, sample_uv) = cubemap_face_scaled_uv_from_direction(direction);
    let sample_x = texel_coord_from_scaled_axis(sample_uv[0], mip_size);
    let sample_y = texel_coord_from_scaled_axis(sample_uv[1], mip_size);
    mip_texel(
        texels,
        face_size,
        mip_count,
        sample_face,
        mip_level,
        sample_x,
        sample_y,
    )
}

fn texel_coord_from_scaled_axis(scaled_axis: Real, face_size: u32) -> u32 {
    (((scaled_axis * 0.5 + 0.5) * face_size as Real - 0.5).round() as i32)
        .clamp(0, face_size.saturating_sub(1) as i32) as u32
}

pub(super) fn mip_texel(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
    x: u32,
    y: u32,
) -> [Real; 4] {
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level);
    texels[offset + y as usize * mip_size as usize + x as usize]
}

fn lerp4(a: [Real; 4], b: [Real; 4], t: Real) -> [Real; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

pub(super) fn normalize_or_positive_z(direction: [Real; 3]) -> [Real; 3] {
    let len_sq =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if len_sq <= Real::EPSILON {
        return [0.0, 0.0, 1.0];
    }
    let inv_len = 1.0 / len_sq.sqrt();
    [
        direction[0] * inv_len,
        direction[1] * inv_len,
        direction[2] * inv_len,
    ]
}
