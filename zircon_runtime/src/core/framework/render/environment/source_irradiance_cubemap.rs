use super::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_texel_direction, cubemap_texel_solid_angle, source_cubemap_face_mip_offset,
    source_cubemap_irradiance_mip_level, source_cubemap_mip_size, CubemapFace,
    SourceCubemapMipChain,
};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;
use std::sync::Arc;

pub use super::ibl_bake_recipe::CANONICAL_IBL_BAKE_IRRADIANCE_CUBE_FACE_SIZE as SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE;

const IRRADIANCE_CUBE_ROWS_PER_TASK: u32 = 4;

struct IrradianceCubeFaceOutput<'a> {
    face: CubemapFace,
    first_row: u32,
    texels: &'a mut [[Real; 3]],
}

trait IrradianceCubeFaceExecutor {
    fn convolve_faces<F>(
        &self,
        face_outputs: &mut [IrradianceCubeFaceOutput<'_>],
        convolve_face: &F,
    ) where
        F: Fn(CubemapFace, u32, &mut [[Real; 3]]) + Send + Sync;
}

struct ParallelIrradianceCubeFaceExecutor<'a, E>(&'a E);

impl<E> IrradianceCubeFaceExecutor for ParallelIrradianceCubeFaceExecutor<'_, E>
where
    E: ParallelSliceExecutor,
{
    fn convolve_faces<F>(
        &self,
        face_outputs: &mut [IrradianceCubeFaceOutput<'_>],
        convolve_face: &F,
    ) where
        F: Fn(CubemapFace, u32, &mut [[Real; 3]]) + Send + Sync,
    {
        self.0.parallel_for(face_outputs, 1, |outputs| {
            for output in outputs {
                convolve_face(output.face, output.first_row, output.texels);
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCubemapIrradianceCube {
    face_size: u32,
    texels: Arc<[[Real; 3]]>,
    content_hash: [u32; 4],
}

impl SourceCubemapIrradianceCube {
    pub fn new(face_size: u32, texels: Vec<[Real; 3]>) -> Self {
        let face_size = face_size.max(1);
        assert_eq!(
            texels.len(),
            source_cubemap_irradiance_cube_sample_count(face_size),
            "source irradiance cubemap texel count must match face size"
        );
        let content_hash = source_cubemap_irradiance_cube_content_hash(face_size, &texels);
        Self {
            face_size,
            texels: texels.into(),
            content_hash,
        }
    }

    pub const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub fn texels(&self) -> &[[Real; 3]] {
        &self.texels
    }

    /// Stable content signature used to invalidate GPU irradiance uploads without per-frame hashing.
    pub const fn content_hash(&self) -> [u32; 4] {
        self.content_hash
    }

    pub fn texel(&self, face: CubemapFace, x: u32, y: u32) -> [Real; 3] {
        let index = source_cubemap_irradiance_cube_face_offset(self.face_size, face)
            + y.min(self.face_size.saturating_sub(1)) as usize * self.face_size as usize
            + x.min(self.face_size.saturating_sub(1)) as usize;
        self.texels[index]
    }
}

fn source_cubemap_irradiance_cube_content_hash(face_size: u32, texels: &[[Real; 3]]) -> [u32; 4] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&face_size.to_le_bytes());
    for texel in texels {
        for channel in texel {
            hasher.update(&channel.to_bits().to_le_bytes());
        }
    }
    let bytes = hasher.finalize();
    let bytes = bytes.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

pub fn build_source_cubemap_irradiance_cube(
    cubemap: &SourceCubemapMipChain,
) -> SourceCubemapIrradianceCube {
    let face_size = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE;
    let mut texels = vec![[0.0; 3]; source_cubemap_irradiance_cube_sample_count(face_size)];
    let source_mip =
        source_cubemap_irradiance_mip_level(cubemap.source_face_size(), cubemap.source_mip_count());
    let face_sample_count = face_size as usize * face_size as usize;

    for face in CubemapFace::ALL {
        let offset = source_cubemap_irradiance_cube_face_offset(face_size, face);
        convolve_irradiance_cube_output_rows(
            cubemap,
            source_mip,
            face,
            0,
            &mut texels[offset..offset + face_sample_count],
            face_size,
        );
    }

    SourceCubemapIrradianceCube::new(face_size, texels)
}

/// Convolves independent irradiance output-row tiles through the caller-owned task executor.
pub fn build_source_cubemap_irradiance_cube_with_parallel_executor<E>(
    cubemap: &SourceCubemapMipChain,
    parallel_executor: &E,
) -> SourceCubemapIrradianceCube
where
    E: ParallelSliceExecutor,
{
    build_source_cubemap_irradiance_cube_with_face_executor(
        cubemap,
        &ParallelIrradianceCubeFaceExecutor(parallel_executor),
    )
}

fn build_source_cubemap_irradiance_cube_with_face_executor(
    cubemap: &SourceCubemapMipChain,
    face_executor: &impl IrradianceCubeFaceExecutor,
) -> SourceCubemapIrradianceCube {
    let face_size = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE;
    let mut texels = vec![[0.0; 3]; source_cubemap_irradiance_cube_sample_count(face_size)];
    let source_mip =
        source_cubemap_irradiance_mip_level(cubemap.source_face_size(), cubemap.source_mip_count());

    let face_sample_count = face_size as usize * face_size as usize;
    let face_output_rows = face_size.div_ceil(IRRADIANCE_CUBE_ROWS_PER_TASK) as usize;
    let mut face_texels = texels.chunks_exact_mut(face_sample_count);
    let mut face_outputs = Vec::with_capacity(CubemapFace::ALL.len() * face_output_rows);
    for face in CubemapFace::ALL {
        let face_texels = face_texels
            .next()
            .expect("irradiance cube storage must contain every cubemap face");
        for (row_chunk_index, texels) in face_texels
            .chunks_mut(face_size as usize * IRRADIANCE_CUBE_ROWS_PER_TASK as usize)
            .enumerate()
        {
            face_outputs.push(IrradianceCubeFaceOutput {
                face,
                first_row: row_chunk_index as u32 * IRRADIANCE_CUBE_ROWS_PER_TASK,
                texels,
            });
        }
    }
    face_executor.convolve_faces(&mut face_outputs, &|face, first_row, output_texels| {
        convolve_irradiance_cube_output_rows(
            cubemap,
            source_mip,
            face,
            first_row,
            output_texels,
            face_size,
        );
    });

    SourceCubemapIrradianceCube::new(face_size, texels)
}

fn convolve_irradiance_cube_output_rows(
    cubemap: &SourceCubemapMipChain,
    source_mip: u32,
    face: CubemapFace,
    first_row: u32,
    output_texels: &mut [[Real; 3]],
    face_size: u32,
) {
    for (row_offset, output_row) in output_texels
        .chunks_exact_mut(face_size as usize)
        .enumerate()
    {
        let y = first_row + row_offset as u32;
        for (x, output_texel) in output_row.iter_mut().enumerate() {
            let normal = cubemap_texel_direction(face, x as u32, y, face_size);
            *output_texel = convolve_source_cubemap_cosine(cubemap, source_mip, normal);
        }
    }
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
    let source_size = source_cubemap_mip_size(cubemap.source_face_size(), source_mip);
    let mut color = [0.0; 3];
    let mut weight_sum = 0.0;

    // Direct cosine convolution produces the optional IEM path without SH band truncation.
    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(
            cubemap.source_face_size(),
            cubemap.source_mip_count(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::environment::build_source_cubemap_from_equirect;
    use crate::core::framework::tasks::ParallelSliceExecutor;

    #[derive(Default)]
    struct CountingParallelSliceExecutor {
        dispatches: std::sync::atomic::AtomicUsize,
        work_items: std::sync::atomic::AtomicUsize,
    }

    impl ParallelSliceExecutor for CountingParallelSliceExecutor {
        fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
        where
            T: Send,
            F: Fn(&mut [T]) + Send + Sync,
        {
            self.dispatches
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.work_items.fetch_add(
                items.len().div_ceil(chunk_size.max(1)),
                std::sync::atomic::Ordering::Relaxed,
            );
            for chunk in items.chunks_mut(chunk_size.max(1)) {
                task(chunk);
            }
        }
    }

    #[test]
    fn cloned_irradiance_cube_shares_immutable_texel_storage() {
        let cube = SourceCubemapIrradianceCube::new(1, vec![[0.25, 0.5, 0.75]; 6]);
        let cloned = cube.clone();

        assert!(std::sync::Arc::ptr_eq(&cube.texels, &cloned.texels));
    }

    #[test]
    fn parallel_irradiance_cube_matches_serial_output_and_tiles_face_rows_for_executor() {
        let source = build_source_cubemap_from_equirect(4, |u, v| {
            [u, v, (u * 0.75 + v * 0.25).fract(), 1.0]
        });
        let serial = build_source_cubemap_irradiance_cube(&source);
        let executor = CountingParallelSliceExecutor::default();
        let parallel =
            build_source_cubemap_irradiance_cube_with_parallel_executor(&source, &executor);

        assert_eq!(parallel, serial);
        assert_eq!(
            executor
                .dispatches
                .load(std::sync::atomic::Ordering::Relaxed),
            1,
            "IEM convolution must submit all independent output rows through one caller-owned executor dispatch"
        );
        assert_eq!(
            executor
                .work_items
                .load(std::sync::atomic::Ordering::Relaxed),
            48,
            "32x32 IEM output must use four-row tiles so a direct convolution can use more than six worker tasks"
        );
    }
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
