use super::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_texel_direction, cubemap_texel_solid_angle, source_cubemap_face_mip_offset,
    source_cubemap_irradiance_mip_level, source_cubemap_mip_size, CubemapFace,
    SourceCubemapMipChain,
};
use crate::core::math::Real;

pub const SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE: u32 = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCubemapIrradianceCube {
    face_size: u32,
    texels: Vec<[Real; 3]>,
}

impl SourceCubemapIrradianceCube {
    pub fn new(face_size: u32, texels: Vec<[Real; 3]>) -> Self {
        let face_size = face_size.max(1);
        assert_eq!(
            texels.len(),
            source_cubemap_irradiance_cube_sample_count(face_size),
            "source irradiance cubemap texel count must match face size"
        );
        Self { face_size, texels }
    }

    pub const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub fn texels(&self) -> &[[Real; 3]] {
        &self.texels
    }

    pub fn texel(&self, face: CubemapFace, x: u32, y: u32) -> [Real; 3] {
        let index = source_cubemap_irradiance_cube_face_offset(self.face_size, face)
            + y.min(self.face_size.saturating_sub(1)) as usize * self.face_size as usize
            + x.min(self.face_size.saturating_sub(1)) as usize;
        self.texels[index]
    }
}

pub fn build_source_cubemap_irradiance_cube(
    cubemap: &SourceCubemapMipChain,
) -> SourceCubemapIrradianceCube {
    let face_size = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE;
    let mut texels = vec![[0.0; 3]; source_cubemap_irradiance_cube_sample_count(face_size)];
    let source_mip = source_cubemap_irradiance_mip_level(cubemap.face_size(), cubemap.mip_count());

    for face in CubemapFace::ALL {
        let offset = source_cubemap_irradiance_cube_face_offset(face_size, face);
        for y in 0..face_size {
            for x in 0..face_size {
                let normal = cubemap_texel_direction(face, x, y, face_size);
                texels[offset + y as usize * face_size as usize + x as usize] =
                    convolve_source_cubemap_cosine(cubemap, source_mip, normal);
            }
        }
    }

    SourceCubemapIrradianceCube::new(face_size, texels)
}

pub fn source_cubemap_sample_irradiance_cube(
    cubemap: &SourceCubemapIrradianceCube,
    normal: [Real; 3],
) -> [Real; 3] {
    sample_irradiance_cube_linear(cubemap, normalize_or_positive_z(normal))
}

fn source_cubemap_irradiance_cube_sample_count(face_size: u32) -> usize {
    let face_size = face_size.max(1) as usize;
    face_size * face_size * CubemapFace::ALL.len()
}

fn source_cubemap_irradiance_cube_face_offset(face_size: u32, face: CubemapFace) -> usize {
    face.index() * face_size as usize * face_size as usize
}

fn convolve_source_cubemap_cosine(
    cubemap: &SourceCubemapMipChain,
    source_mip: u32,
    normal: [Real; 3],
) -> [Real; 3] {
    let source_size = source_cubemap_mip_size(cubemap.face_size(), source_mip);
    let mut color = [0.0; 3];
    let mut weight_sum = 0.0;

    // Direct cosine convolution produces the optional IEM path without SH band truncation.
    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(
            cubemap.face_size(),
            cubemap.mip_count(),
            face,
            source_mip,
        );
        for y in 0..source_size {
            for x in 0..source_size {
                let direction = cubemap_texel_direction(face, x, y, source_size);
                let no_l = dot3(normal, direction).max(0.0);
                if no_l <= 0.0 {
                    continue;
                }
                let weight = no_l * cubemap_texel_solid_angle(x, y, source_size);
                let texel = cubemap.source_texels()
                    [offset + y as usize * source_size as usize + x as usize];
                color[0] += texel[0] * weight;
                color[1] += texel[1] * weight;
                color[2] += texel[2] * weight;
                weight_sum += weight;
            }
        }
    }

    if weight_sum <= Real::EPSILON {
        return [0.0; 3];
    }
    [
        color[0] / weight_sum,
        color[1] / weight_sum,
        color[2] / weight_sum,
    ]
}

fn sample_irradiance_cube_linear(
    cubemap: &SourceCubemapIrradianceCube,
    direction: [Real; 3],
) -> [Real; 3] {
    let (face, scaled_uv) = cubemap_face_scaled_uv_from_direction(direction);
    let face_size = cubemap.face_size();
    let texel_x = (scaled_uv[0] * 0.5 + 0.5) * face_size as Real - 0.5;
    let texel_y = (scaled_uv[1] * 0.5 + 0.5) * face_size as Real - 0.5;
    let x0 = texel_x.floor();
    let y0 = texel_y.floor();
    let tx = texel_x - x0;
    let ty = texel_y - y0;
    let x0 = x0 as i32;
    let y0 = y0 as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    lerp3(
        lerp3(
            sample_irradiance_cube_texel_unwrapped(cubemap, face, x0, y0),
            sample_irradiance_cube_texel_unwrapped(cubemap, face, x1, y0),
            tx,
        ),
        lerp3(
            sample_irradiance_cube_texel_unwrapped(cubemap, face, x0, y1),
            sample_irradiance_cube_texel_unwrapped(cubemap, face, x1, y1),
            tx,
        ),
        ty,
    )
}

fn sample_irradiance_cube_texel_unwrapped(
    cubemap: &SourceCubemapIrradianceCube,
    face: CubemapFace,
    x: i32,
    y: i32,
) -> [Real; 3] {
    let face_size = cubemap.face_size();
    let face_size_i32 = face_size as i32;
    if x >= 0 && x < face_size_i32 && y >= 0 && y < face_size_i32 {
        return cubemap.texel(face, x as u32, y as u32);
    }

    let scaled_uv = [
        ((x as Real + 0.5) / face_size as Real) * 2.0 - 1.0,
        ((y as Real + 0.5) / face_size as Real) * 2.0 - 1.0,
    ];
    let direction = cubemap_direction_from_scaled_uv(face, scaled_uv);
    let (sample_face, sample_uv) = cubemap_face_scaled_uv_from_direction(direction);
    cubemap.texel(
        sample_face,
        texel_coord_from_scaled_axis(sample_uv[0], face_size),
        texel_coord_from_scaled_axis(sample_uv[1], face_size),
    )
}

fn texel_coord_from_scaled_axis(scaled_axis: Real, face_size: u32) -> u32 {
    (((scaled_axis * 0.5 + 0.5) * face_size as Real - 0.5).round() as i32)
        .clamp(0, face_size.saturating_sub(1) as i32) as u32
}

fn lerp3(a: [Real; 3], b: [Real; 3], t: Real) -> [Real; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn dot3(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize_or_positive_z(direction: [Real; 3]) -> [Real; 3] {
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
