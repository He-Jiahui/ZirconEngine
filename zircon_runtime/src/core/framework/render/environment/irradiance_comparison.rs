use super::{
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_scaled_uv_for_texel, CubemapFace, SourceCubemapIrradianceCube,
};
use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCubemapIrradianceComparisonError {
    FaceSizeMismatch { reference: u32, candidate: u32 },
}

/// Per-channel absolute-error statistics for comparing two irradiance cubemaps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceCubemapIrradianceErrorStatistics {
    sample_count: usize,
    edge_sample_count: usize,
    seam_sample_count: usize,
    mean_absolute_error: [Real; 3],
    max_absolute_error: [Real; 3],
    edge_mean_absolute_error: [Real; 3],
    edge_max_absolute_error: [Real; 3],
    seam_mean_absolute_error: [Real; 3],
    seam_max_absolute_error: [Real; 3],
}

impl SourceCubemapIrradianceErrorStatistics {
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub const fn edge_sample_count(&self) -> usize {
        self.edge_sample_count
    }

    pub const fn seam_sample_count(&self) -> usize {
        self.seam_sample_count
    }

    pub const fn mean_absolute_error(&self) -> [Real; 3] {
        self.mean_absolute_error
    }

    pub const fn max_absolute_error(&self) -> [Real; 3] {
        self.max_absolute_error
    }

    pub const fn edge_mean_absolute_error(&self) -> [Real; 3] {
        self.edge_mean_absolute_error
    }

    pub const fn edge_max_absolute_error(&self) -> [Real; 3] {
        self.edge_max_absolute_error
    }

    pub const fn seam_mean_absolute_error(&self) -> [Real; 3] {
        self.seam_mean_absolute_error
    }

    pub const fn seam_max_absolute_error(&self) -> [Real; 3] {
        self.seam_max_absolute_error
    }
}

/// 在 CPU 上比较两个同尺寸源辐照度立方体，并分别统计整体、边缘和接缝误差；该函数不执行 GPU 烘焙或回读。
pub fn compare_source_cubemap_irradiance(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> Result<SourceCubemapIrradianceErrorStatistics, SourceCubemapIrradianceComparisonError> {
    let face_size = reference.face_size();
    if candidate.face_size() != face_size {
        return Err(SourceCubemapIrradianceComparisonError::FaceSizeMismatch {
            reference: face_size,
            candidate: candidate.face_size(),
        });
    }

    let face_size = face_size as usize;
    let face_sample_count = face_size * face_size;
    let mut total_error: [Real; 3] = [0.0; 3];
    let mut max_error: [Real; 3] = [0.0; 3];
    let mut edge_error: [Real; 3] = [0.0; 3];
    let mut edge_max_error: [Real; 3] = [0.0; 3];
    let mut edge_sample_count = 0_usize;

    for (reference_face, candidate_face) in reference
        .texels()
        .chunks_exact(face_sample_count)
        .zip(candidate.texels().chunks_exact(face_sample_count))
    {
        for (y, (reference_row, candidate_row)) in reference_face
            .chunks_exact(face_size)
            .zip(candidate_face.chunks_exact(face_size))
            .enumerate()
        {
            let row_is_edge = y == 0 || y + 1 == face_size;
            for (x, (reference, candidate)) in reference_row.iter().zip(candidate_row).enumerate() {
                let is_edge = row_is_edge || x == 0 || x + 1 == face_size;
                if is_edge {
                    edge_sample_count += 1;
                }
                for channel in 0..3 {
                    let error = (candidate[channel] - reference[channel]).abs();
                    total_error[channel] += error;
                    max_error[channel] = max_error[channel].max(error);
                    if is_edge {
                        edge_error[channel] += error;
                        edge_max_error[channel] = edge_max_error[channel].max(error);
                    }
                }
            }
        }
    }

    let sample_count = reference.texels().len();
    let sample_count_real = sample_count.max(1) as Real;
    let edge_sample_count_real = edge_sample_count.max(1) as Real;
    let (seam_error, seam_max_error, seam_sample_count) =
        compare_cubemap_seam_error(reference, candidate);
    let seam_sample_count_real = seam_sample_count.max(1) as Real;
    Ok(SourceCubemapIrradianceErrorStatistics {
        sample_count,
        edge_sample_count,
        seam_sample_count,
        mean_absolute_error: divide_channels(total_error, sample_count_real),
        max_absolute_error: max_error,
        edge_mean_absolute_error: divide_channels(edge_error, edge_sample_count_real),
        edge_max_absolute_error: edge_max_error,
        seam_mean_absolute_error: divide_channels(seam_error, seam_sample_count_real),
        seam_max_absolute_error: seam_max_error,
    })
}

fn compare_cubemap_seam_error(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> ([Real; 3], [Real; 3], usize) {
    let face_size = reference.face_size();
    let mut total_error: [Real; 3] = [0.0; 3];
    let mut max_error: [Real; 3] = [0.0; 3];
    let mut sample_count = 0_usize;

    for face in CubemapFace::ALL {
        for edge in CubemapEdge::ALL {
            for index in 0..face_size {
                let (x, y) = edge.texel(index, face_size);
                let (neighbor_face, neighbor_x, neighbor_y) =
                    cubemap_seam_neighbor(face, edge, index, face_size);
                let reference_delta = subtract_channels(
                    reference.texel(face, x, y),
                    reference.texel(neighbor_face, neighbor_x, neighbor_y),
                );
                let candidate_delta = subtract_channels(
                    candidate.texel(face, x, y),
                    candidate.texel(neighbor_face, neighbor_x, neighbor_y),
                );
                for channel in 0..3 {
                    let error = (candidate_delta[channel] - reference_delta[channel]).abs();
                    total_error[channel] += error;
                    max_error[channel] = max_error[channel].max(error);
                }
                sample_count += 1;
            }
        }
    }

    (total_error, max_error, sample_count)
}

fn cubemap_seam_neighbor(
    face: CubemapFace,
    edge: CubemapEdge,
    index: u32,
    face_size: u32,
) -> (CubemapFace, u32, u32) {
    let edge_uv = edge.outside_scaled_uv(index, face_size);
    let direction = cubemap_direction_from_scaled_uv(face, edge_uv);
    let (neighbor_face, neighbor_uv) = cubemap_face_scaled_uv_from_direction(direction);
    (
        neighbor_face,
        scaled_axis_to_texel(neighbor_uv[0], face_size),
        scaled_axis_to_texel(neighbor_uv[1], face_size),
    )
}

fn scaled_axis_to_texel(scaled_axis: Real, face_size: u32) -> u32 {
    (((scaled_axis * 0.5 + 0.5) * face_size as Real - 0.5).round() as i32)
        .clamp(0, face_size.saturating_sub(1) as i32) as u32
}

fn subtract_channels(first: [Real; 3], second: [Real; 3]) -> [Real; 3] {
    [
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    ]
}

#[derive(Clone, Copy)]
enum CubemapEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl CubemapEdge {
    const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    fn texel(self, index: u32, face_size: u32) -> (u32, u32) {
        match self {
            Self::Left => (0, index),
            Self::Right => (face_size.saturating_sub(1), index),
            Self::Top => (index, 0),
            Self::Bottom => (index, face_size.saturating_sub(1)),
        }
    }

    fn outside_scaled_uv(self, index: u32, face_size: u32) -> [Real; 2] {
        match self {
            Self::Left => [
                -1.0 - 1.0 / face_size.max(1) as Real,
                cubemap_scaled_uv_for_texel(0, index, face_size)[1],
            ],
            Self::Right => [
                1.0 + 1.0 / face_size.max(1) as Real,
                cubemap_scaled_uv_for_texel(face_size.saturating_sub(1), index, face_size)[1],
            ],
            Self::Top => [
                cubemap_scaled_uv_for_texel(index, 0, face_size)[0],
                -1.0 - 1.0 / face_size.max(1) as Real,
            ],
            Self::Bottom => [
                cubemap_scaled_uv_for_texel(index, face_size.saturating_sub(1), face_size)[0],
                1.0 + 1.0 / face_size.max(1) as Real,
            ],
        }
    }
}

fn divide_channels(values: [Real; 3], divisor: Real) -> [Real; 3] {
    [
        values[0] / divisor,
        values[1] / divisor,
        values[2] / divisor,
    ]
}

#[cfg(test)]
#[path = "tests/irradiance_comparison.rs"]
mod tests;
