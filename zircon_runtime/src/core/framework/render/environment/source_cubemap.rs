use super::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_face_size_from_equirect_height, cubemap_texel_direction, cubemap_texel_solid_angle,
    equirect_uv_from_direction, CubemapFace,
};
use crate::core::math::Real;

mod mipmap;
mod pmrem;

pub const SOURCE_CUBEMAP_FACE_COUNT: usize = 6;
pub const SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT: usize = 9;
pub const SOURCE_CUBEMAP_IRRADIANCE_SOURCE_FACE_SIZE: u32 = 32;
pub const SOURCE_CUBEMAP_MIN_FACE_SIZE: u32 = 64;
pub const SOURCE_CUBEMAP_MAX_FACE_SIZE: u32 = 1024;
pub const SOURCE_CUBEMAP_ROUGHEST_MIP: u32 = 10;
pub const SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE: Real = SOURCE_CUBEMAP_ROUGHEST_MIP as Real;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceCubemapPrefilterQuality {
    Fast,
    #[default]
    Normal,
    High,
}

pub type SourceCubemapIrradianceSh9 = [[Real; 4]; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT];

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCubemapMipChain {
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    texels: Vec<[Real; 4]>,
    irradiance_sh9: SourceCubemapIrradianceSh9,
}

impl SourceCubemapMipChain {
    pub fn new(face_size: u32, mip_count: u32, texels: Vec<[Real; 4]>) -> Self {
        let face_size = face_size.max(1);
        let mip_count = mip_count.max(1);
        assert_eq!(
            texels.len(),
            source_cubemap_sample_count(face_size, mip_count),
            "source cubemap texel count must match face size and mip count"
        );
        let source_texels = texels.clone();
        let irradiance_sh9 =
            source_cubemap_irradiance_sh9_from_texels(&source_texels, face_size, mip_count, 0);
        Self::new_with_source_texels_and_irradiance_sh9(
            face_size,
            mip_count,
            source_texels,
            texels,
            irradiance_sh9,
        )
    }

    pub(super) fn new_with_source_texels_and_irradiance_sh9(
        face_size: u32,
        mip_count: u32,
        source_texels: Vec<[Real; 4]>,
        texels: Vec<[Real; 4]>,
        irradiance_sh9: SourceCubemapIrradianceSh9,
    ) -> Self {
        assert_eq!(
            source_texels.len(),
            source_cubemap_sample_count(face_size, mip_count),
            "source cubemap regular texel count must match face size and mip count"
        );
        Self {
            face_size,
            mip_count,
            source_texels,
            texels,
            irradiance_sh9,
        }
    }

    pub const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub fn source_texels(&self) -> &[[Real; 4]] {
        &self.source_texels
    }

    pub fn texels(&self) -> &[[Real; 4]] {
        &self.texels
    }

    pub fn irradiance_sh9(&self) -> &SourceCubemapIrradianceSh9 {
        &self.irradiance_sh9
    }

    pub fn texel(&self, face: CubemapFace, mip_level: u32, x: u32, y: u32) -> [Real; 4] {
        let mip_size = source_cubemap_mip_size(self.face_size, mip_level);
        let index = source_cubemap_face_mip_offset(self.face_size, self.mip_count, face, mip_level)
            + y.min(mip_size.saturating_sub(1)) as usize * mip_size as usize
            + x.min(mip_size.saturating_sub(1)) as usize;
        self.texels[index]
    }
}

pub fn source_cubemap_face_size_from_equirect_height(equirect_height: u32) -> u32 {
    cubemap_face_size_from_equirect_height(equirect_height)
        .next_power_of_two()
        .clamp(SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_MAX_FACE_SIZE)
}

pub fn source_cubemap_mip_count(face_size: u32) -> u32 {
    let mut size = face_size.max(1);
    let mut count = 1;
    while size > 1 {
        size = (size / 2).max(1);
        count += 1;
    }
    count
}

pub fn source_cubemap_mip_size(face_size: u32, mip_level: u32) -> u32 {
    let shifted = face_size.max(1) >> mip_level.min(u32::BITS - 1);
    shifted.max(1)
}

pub fn source_cubemap_sample_count(face_size: u32, mip_count: u32) -> usize {
    let per_face = source_cubemap_samples_per_face(face_size, mip_count);
    per_face * SOURCE_CUBEMAP_FACE_COUNT
}

pub fn source_cubemap_capture_hash(
    face_size: u32,
    face_major_base_texels: &[[Real; 4]],
) -> [u32; 4] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&face_size.max(1).to_le_bytes());
    for texel in face_major_base_texels {
        for channel in texel {
            hasher.update(&channel.to_bits().to_le_bytes());
        }
    }
    let hash = hasher.finalize();
    let bytes = hash.as_bytes();
    [
        u32::from_le_bytes(bytes[0..4].try_into().expect("blake3 word 0")),
        u32::from_le_bytes(bytes[4..8].try_into().expect("blake3 word 1")),
        u32::from_le_bytes(bytes[8..12].try_into().expect("blake3 word 2")),
        u32::from_le_bytes(bytes[12..16].try_into().expect("blake3 word 3")),
    ]
}

pub fn source_cubemap_face_mip_offset(
    face_size: u32,
    mip_count: u32,
    face: CubemapFace,
    mip_level: u32,
) -> usize {
    let mip_level = mip_level.min(mip_count.saturating_sub(1));
    face.index() * source_cubemap_samples_per_face(face_size, mip_count)
        + source_cubemap_mip_offset_within_face(face_size, mip_level)
}

pub fn build_source_cubemap_from_equirect<F>(
    face_size: u32,
    mut sample_equirect: F,
) -> SourceCubemapMipChain
where
    F: FnMut(Real, Real) -> [Real; 4],
{
    let face_size = face_size.max(1);
    let mip_count = source_cubemap_mip_count(face_size);
    let mut texels = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];

    for face in CubemapFace::ALL {
        let base_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
        for y in 0..face_size {
            for x in 0..face_size {
                let direction = cubemap_texel_direction(face, x, y, face_size);
                let uv = equirect_uv_from_direction(direction);
                texels[base_offset + y as usize * face_size as usize + x as usize] =
                    sample_equirect(uv[0], uv[1]);
            }
        }
    }

    // Source mips stay separate from the PMREM chain: skybox minification and FIS
    // source LOD read this angular-filtered pyramid, while reflections read PMREM.
    let source_mips = mipmap::source_cubemap_mips_from_base(&texels, face_size, mip_count);
    let irradiance_sh9 = source_cubemap_irradiance_sh9_from_texels(
        &source_mips,
        face_size,
        mip_count,
        source_cubemap_irradiance_mip_level(face_size, mip_count),
    );
    pmrem::prefilter_pmrem_mips_from_source(
        &mut texels,
        &source_mips,
        face_size,
        mip_count,
        SourceCubemapPrefilterQuality::Normal,
    );

    average_last_mip_faces(&mut texels, face_size, mip_count);
    SourceCubemapMipChain::new_with_source_texels_and_irradiance_sh9(
        face_size,
        mip_count,
        source_mips,
        texels,
        irradiance_sh9,
    )
}

/// Builds the source mip pyramid, PMREM chain, and SH9 data from six captured
/// face images in cmft face order. Input contains only mip zero, face-major.
pub fn build_source_cubemap_from_captured_faces(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
) -> SourceCubemapMipChain {
    build_source_cubemap_from_captured_faces_with_quality(
        face_size,
        captured_face_texels,
        SourceCubemapPrefilterQuality::Normal,
    )
}

pub fn build_source_cubemap_from_captured_faces_with_quality(
    face_size: u32,
    captured_face_texels: Vec<[Real; 4]>,
    quality: SourceCubemapPrefilterQuality,
) -> SourceCubemapMipChain {
    let face_size = face_size.max(1);
    let expected_base_texel_count =
        face_size as usize * face_size as usize * SOURCE_CUBEMAP_FACE_COUNT;
    assert_eq!(
        captured_face_texels.len(),
        expected_base_texel_count,
        "captured cubemap must contain six face-major mip-zero images"
    );

    let mip_count = source_cubemap_mip_count(face_size);
    let mut source_storage = vec![[0.0; 4]; source_cubemap_sample_count(face_size, mip_count)];
    let face_texel_count = face_size as usize * face_size as usize;
    for face in CubemapFace::ALL {
        let source_offset = face.index() * face_texel_count;
        let target_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, 0);
        source_storage[target_offset..target_offset + face_texel_count].copy_from_slice(
            &captured_face_texels[source_offset..source_offset + face_texel_count],
        );
    }

    let source_mips = mipmap::source_cubemap_mips_from_base(&source_storage, face_size, mip_count);
    build_source_cubemap_from_source_mips_with_quality(face_size, mip_count, source_mips, quality)
}

/// Rebuild Zircon PMREM/SH9 from an external cubemap's source mip pyramid.
pub fn build_source_cubemap_from_source_mips(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
) -> SourceCubemapMipChain {
    build_source_cubemap_from_source_mips_with_quality(
        face_size,
        mip_count,
        source_texels,
        SourceCubemapPrefilterQuality::Normal,
    )
}

pub fn build_source_cubemap_from_source_mips_with_quality(
    face_size: u32,
    mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    quality: SourceCubemapPrefilterQuality,
) -> SourceCubemapMipChain {
    let face_size = face_size.max(1);
    let mip_count = mip_count.max(1);
    assert_eq!(
        source_texels.len(),
        source_cubemap_sample_count(face_size, mip_count),
        "external source cubemap texel count must match face size and mip count"
    );
    let irradiance_sh9 = source_cubemap_irradiance_sh9_from_texels(
        &source_texels,
        face_size,
        mip_count,
        source_cubemap_irradiance_mip_level(face_size, mip_count),
    );
    let mut pmrem_texels = source_texels.clone();
    pmrem::prefilter_pmrem_mips_from_source(
        &mut pmrem_texels,
        &source_texels,
        face_size,
        mip_count,
        quality,
    );
    average_last_mip_faces(&mut pmrem_texels, face_size, mip_count);
    SourceCubemapMipChain::new_with_source_texels_and_irradiance_sh9(
        face_size,
        mip_count,
        source_texels,
        pmrem_texels,
        irradiance_sh9,
    )
}

pub fn source_cubemap_pmrem_mip_from_roughness(roughness: Real, mip_count: u32) -> Real {
    let max_mip = mip_count.max(1) as Real - 1.0;
    roughness.clamp(0.0, 1.0) * max_mip
}

pub fn source_cubemap_roughness_from_pmrem_mip(mip_level: u32, mip_count: u32) -> Real {
    let max_mip = mip_count.max(1).saturating_sub(1);
    if max_mip == 0 {
        return 0.0;
    }
    mip_level.min(max_mip) as Real / max_mip as Real
}

pub fn source_cubemap_irradiance_mip_level(face_size: u32, mip_count: u32) -> u32 {
    let mut mip_level = 0;
    let mip_count = mip_count.max(1);
    while mip_level + 1 < mip_count
        && source_cubemap_mip_size(face_size, mip_level)
            > SOURCE_CUBEMAP_IRRADIANCE_SOURCE_FACE_SIZE
    {
        mip_level += 1;
    }
    mip_level
}

pub fn source_cubemap_evaluate_irradiance_sh9(
    coefficients: &SourceCubemapIrradianceSh9,
    normal: [Real; 3],
) -> [Real; 3] {
    let basis = sh9_basis_y_up(normalize_or_positive_z(normal));
    let mut irradiance = [0.0; 3];
    for coefficient_index in 0..SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT {
        irradiance[0] += coefficients[coefficient_index][0] * basis[coefficient_index];
        irradiance[1] += coefficients[coefficient_index][1] * basis[coefficient_index];
        irradiance[2] += coefficients[coefficient_index][2] * basis[coefficient_index];
    }
    [
        irradiance[0].max(0.0),
        irradiance[1].max(0.0),
        irradiance[2].max(0.0),
    ]
}

fn source_cubemap_samples_per_face(face_size: u32, mip_count: u32) -> usize {
    let mut total = 0;
    for mip in 0..mip_count.max(1) {
        let size = source_cubemap_mip_size(face_size, mip);
        total += size as usize * size as usize;
    }
    total
}

fn source_cubemap_mip_offset_within_face(face_size: u32, mip_level: u32) -> usize {
    let mut offset = 0;
    for mip in 0..mip_level {
        let size = source_cubemap_mip_size(face_size, mip);
        offset += size as usize * size as usize;
    }
    offset
}

fn source_cubemap_irradiance_sh9_from_texels(
    texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
) -> SourceCubemapIrradianceSh9 {
    let mip_level = mip_level.min(mip_count.max(1).saturating_sub(1));
    let mip_size = source_cubemap_mip_size(face_size, mip_level);
    let mut coefficients = [[0.0; 4]; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT];
    let mut solid_angle_sum = 0.0;

    for face in CubemapFace::ALL {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let direction = cubemap_texel_direction(face, x, y, mip_size);
                let solid_angle = cubemap_texel_solid_angle(x, y, mip_size);
                let texel = mip_texel(texels, face_size, mip_count, face, mip_level, x, y);
                let basis = sh9_basis_y_up(direction);
                solid_angle_sum += solid_angle;
                for coefficient_index in 0..SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT {
                    let weighted_basis = basis[coefficient_index] * solid_angle;
                    coefficients[coefficient_index][0] += texel[0] * weighted_basis;
                    coefficients[coefficient_index][1] += texel[1] * weighted_basis;
                    coefficients[coefficient_index][2] += texel[2] * weighted_basis;
                }
            }
        }
    }

    let normalization = std::f32::consts::TAU * 2.0 / solid_angle_sum.max(Real::EPSILON);
    for (coefficient_index, coefficient) in coefficients.iter_mut().enumerate() {
        let band_scale = sh9_cosine_lobe_scale(coefficient_index);
        coefficient[0] *= normalization * band_scale;
        coefficient[1] *= normalization * band_scale;
        coefficient[2] *= normalization * band_scale;
    }

    coefficients
}

fn sh9_basis_y_up(direction: [Real; 3]) -> [Real; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT] {
    let x = direction[0];
    let y = direction[1];
    let z = direction[2];
    [
        0.282_094_8,
        0.488_602_52 * z,
        0.488_602_52 * y,
        0.488_602_52 * x,
        1.092_548_5 * x * z,
        1.092_548_5 * z * y,
        0.315_391_57 * (3.0 * y * y - 1.0),
        1.092_548_5 * x * y,
        0.546_274_24 * (x * x - z * z),
    ]
}

fn sh9_cosine_lobe_scale(coefficient_index: usize) -> Real {
    match coefficient_index {
        0 => 1.0,
        1..=3 => 2.0 / 3.0,
        _ => 0.25,
    }
}

fn sample_source_cubemap_trilinear(
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

fn sample_cubemap_linear_at_mip(
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

fn mip_texel(
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

fn tangent_basis(direction: [Real; 3]) -> [[Real; 3]; 3] {
    let normal = normalize_or_positive_z(direction);
    let up = if normal[1].abs() < 0.999 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let tangent = normalize_or_positive_z(cross3(up, normal));
    let bitangent = cross3(normal, tangent);
    [tangent, bitangent, normal]
}

fn cross3(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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

fn average_last_mip_faces(texels: &mut [[Real; 4]], face_size: u32, mip_count: u32) {
    let last_mip = mip_count.saturating_sub(1);
    if source_cubemap_mip_size(face_size, last_mip) != 1 {
        return;
    }

    let mut average = [0.0; 4];
    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, last_mip);
        let texel = texels[offset];
        average[0] += texel[0];
        average[1] += texel[1];
        average[2] += texel[2];
        average[3] += texel[3];
    }
    let inv_face_count = 1.0 / SOURCE_CUBEMAP_FACE_COUNT as Real;
    average[0] *= inv_face_count;
    average[1] *= inv_face_count;
    average[2] *= inv_face_count;
    average[3] *= inv_face_count;

    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, last_mip);
        texels[offset] = average;
    }
}

#[cfg(test)]
mod tests;
