use super::{
    average_last_mip_faces, sample_cubemap_linear_at_mip, source_cubemap_face_mip_offset,
    source_cubemap_mip_size,
};
use crate::core::framework::render::environment::{
    cubemap_direction_from_scaled_uv, cubemap_scaled_uv_for_texel,
    cubemap_solid_angle_from_scaled_uv, cubemap_texel_direction, CubemapFace,
};
use crate::core::math::Real;

const SOURCE_MIPMAP_SAMPLE_GRID: u32 = 4;

pub(super) fn source_cubemap_mips_from_base(
    base_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
) -> Vec<[Real; 4]> {
    let mut average_mips = base_texels.to_vec();
    build_average_mip_pyramid(&mut average_mips, face_size, mip_count);

    let mut source_mips = average_mips.clone();
    for mip in 1..mip_count {
        filter_source_mip_from_angular_footprint(
            &mut source_mips,
            &average_mips,
            face_size,
            mip_count,
            mip,
        );
    }
    average_last_mip_faces(&mut source_mips, face_size, mip_count);
    source_mips
}

fn build_average_mip_pyramid(texels: &mut [[Real; 4]], face_size: u32, mip_count: u32) {
    for mip in 1..mip_count {
        let mip_size = source_cubemap_mip_size(face_size, mip);
        let previous_mip = mip - 1;
        let previous_size = source_cubemap_mip_size(face_size, previous_mip);
        for face in CubemapFace::ALL {
            let dest_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
            for y in 0..mip_size {
                for x in 0..mip_size {
                    let mut color = [0.0; 4];
                    for child_y in 0..2 {
                        for child_x in 0..2 {
                            let direction = cubemap_texel_direction(
                                face,
                                (x * 2 + child_x).min(previous_size.saturating_sub(1)),
                                (y * 2 + child_y).min(previous_size.saturating_sub(1)),
                                previous_size,
                            );
                            let sample = sample_cubemap_linear_at_mip(
                                texels,
                                face_size,
                                mip_count,
                                direction,
                                previous_mip,
                            );
                            add_weighted(&mut color, sample, 0.25);
                        }
                    }
                    texels[dest_offset + y as usize * mip_size as usize + x as usize] = color;
                }
            }
        }
    }
}

fn filter_source_mip_from_angular_footprint(
    output_mips: &mut [[Real; 4]],
    average_mips: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    mip: u32,
) {
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let input_mip = mip.saturating_sub(1);
    let sample_grid = SOURCE_MIPMAP_SAMPLE_GRID.max(1);
    let subcell_half_width = 1.0 / (mip_size.max(1) * sample_grid) as Real;
    let texel_half_width = 1.0 / mip_size.max(1) as Real;
    let subcell_step = texel_half_width * 2.0 / sample_grid as Real;

    for face in CubemapFace::ALL {
        let dest_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
        for y in 0..mip_size {
            for x in 0..mip_size {
                let center_uv = cubemap_scaled_uv_for_texel(x, y, mip_size);
                let mut color = [0.0; 4];
                let mut weight_sum = 0.0;

                for sample_y in 0..sample_grid {
                    for sample_x in 0..sample_grid {
                        let sample_uv = [
                            center_uv[0] - texel_half_width
                                + (sample_x as Real + 0.5) * subcell_step,
                            center_uv[1] - texel_half_width
                                + (sample_y as Real + 0.5) * subcell_step,
                        ];
                        let direction = cubemap_direction_from_scaled_uv(face, sample_uv);
                        let sample = sample_cubemap_linear_at_mip(
                            average_mips,
                            face_size,
                            mip_count,
                            direction,
                            input_mip,
                        );
                        let weight =
                            cubemap_solid_angle_from_scaled_uv(sample_uv, subcell_half_width)
                                .max(Real::EPSILON);
                        add_weighted(&mut color, sample, weight);
                        weight_sum += weight;
                    }
                }

                let inv_weight = 1.0 / weight_sum.max(Real::EPSILON);
                output_mips[dest_offset + y as usize * mip_size as usize + x as usize] = [
                    color[0] * inv_weight,
                    color[1] * inv_weight,
                    color[2] * inv_weight,
                    color[3] * inv_weight,
                ];
            }
        }
    }
}

fn add_weighted(accumulator: &mut [Real; 4], texel: [Real; 4], weight: Real) {
    accumulator[0] += texel[0] * weight;
    accumulator[1] += texel[1] * weight;
    accumulator[2] += texel[2] * weight;
    accumulator[3] += texel[3] * weight;
}
