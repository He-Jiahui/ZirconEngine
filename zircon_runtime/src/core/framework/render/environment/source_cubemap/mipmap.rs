use super::super::cubemap_projection::cubemap_side_space_direction;
use super::{
    average_last_mip_faces, mip_texel, sample_cubemap_linear_at_mip,
    source_cubemap_face_mip_offset, source_cubemap_face_mip_outputs, source_cubemap_mip_size,
    CubemapFaceMipOutput,
};
use crate::core::framework::render::environment::{
    cubemap_texel_direction, cubemap_texel_solid_angle, CubemapFace,
};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;

const SOURCE_MIPMAP_MIN_CONE_ANGLE: Real = 0.002;
const SOURCE_MIPMAP_MAX_CONE_ANGLE: Real = std::f32::consts::FRAC_PI_2;
const SOURCE_MIPMAP_INPUT_QUALITY_BIAS: Real = 3.0;
const SOURCE_MIPMAP_NORMALIZED_SPHERE_RADIUS: Real = 0.282_094_78;
const SOURCE_MIPMAP_NODE_RADIUS_SCALE: Real = 2.0;

trait SourceMipmapFaceExecutor {
    fn filter_faces<F>(&self, faces: &mut [CubemapFaceMipOutput<'_>], filter_face: &F)
    where
        F: Fn(CubemapFace, &mut [[Real; 4]]) + Send + Sync;
}

struct SerialSourceMipmapFaceExecutor;

impl SourceMipmapFaceExecutor for SerialSourceMipmapFaceExecutor {
    fn filter_faces<F>(&self, faces: &mut [CubemapFaceMipOutput<'_>], filter_face: &F)
    where
        F: Fn(CubemapFace, &mut [[Real; 4]]) + Send + Sync,
    {
        for output in faces {
            filter_face(output.face, &mut *output.texels);
        }
    }
}

struct ParallelSourceMipmapFaceExecutor<'a, E>(&'a E);

impl<E> SourceMipmapFaceExecutor for ParallelSourceMipmapFaceExecutor<'_, E>
where
    E: ParallelSliceExecutor,
{
    fn filter_faces<F>(&self, faces: &mut [CubemapFaceMipOutput<'_>], filter_face: &F)
    where
        F: Fn(CubemapFace, &mut [[Real; 4]]) + Send + Sync,
    {
        self.0.parallel_for(faces, 1, |chunk| {
            for output in chunk {
                filter_face(output.face, &mut *output.texels);
            }
        });
    }
}

pub(super) fn source_cubemap_mips_from_base(
    base_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
) -> Vec<[Real; 4]> {
    source_cubemap_mips_from_base_with_face_executor(
        base_texels,
        face_size,
        mip_count,
        &SerialSourceMipmapFaceExecutor,
    )
}

pub(super) fn source_cubemap_mips_from_base_with_parallel_executor<E>(
    base_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    parallel_executor: &E,
) -> Vec<[Real; 4]>
where
    E: ParallelSliceExecutor,
{
    source_cubemap_mips_from_base_with_face_executor(
        base_texels,
        face_size,
        mip_count,
        &ParallelSourceMipmapFaceExecutor(parallel_executor),
    )
}

fn source_cubemap_mips_from_base_with_face_executor<E>(
    base_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    face_executor: &E,
) -> Vec<[Real; 4]>
where
    E: SourceMipmapFaceExecutor,
{
    let average_mips = source_cubemap_average_mips_from_base(base_texels, face_size, mip_count);

    let mut source_mips = average_mips.clone();
    let mut solid_angle_tables = vec![None; mip_count as usize];
    for mip in 1..mip_count {
        filter_source_mip_from_angular_footprint(
            &mut source_mips,
            &average_mips,
            &mut solid_angle_tables,
            face_size,
            mip_count,
            mip,
            face_executor,
        );
    }
    average_last_mip_faces(&mut source_mips, face_size, mip_count);
    source_mips
}

pub(super) fn source_cubemap_average_mips_from_base(
    base_texels: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
) -> Vec<[Real; 4]> {
    let mut average_mips = base_texels.to_vec();
    build_average_mip_pyramid(&mut average_mips, face_size, mip_count);
    average_mips
}

fn build_average_mip_pyramid(texels: &mut [[Real; 4]], face_size: u32, mip_count: u32) {
    for mip in 1..mip_count {
        let mip_size = source_cubemap_mip_size(face_size, mip);
        let previous_mip = mip - 1;
        let previous_size = source_cubemap_mip_size(face_size, previous_mip);
        for face in CubemapFace::ALL {
            let dest_offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip);
            for y in 0..mip_size {
                let source_y_start = y * previous_size / mip_size;
                let source_y_end = ((y + 1) * previous_size / mip_size)
                    .max(source_y_start + 1)
                    .min(previous_size);
                for x in 0..mip_size {
                    let source_x_start = x * previous_size / mip_size;
                    let source_x_end = ((x + 1) * previous_size / mip_size)
                        .max(source_x_start + 1)
                        .min(previous_size);
                    let mut color = [0.0; 4];
                    let sample_count = ((source_x_end - source_x_start)
                        * (source_y_end - source_y_start))
                        .max(1) as Real;
                    for source_y in source_y_start..source_y_end {
                        for source_x in source_x_start..source_x_end {
                            let sample = mip_texel(
                                texels,
                                face_size,
                                mip_count,
                                face,
                                previous_mip,
                                source_x,
                                source_y,
                            );
                            add_weighted(&mut color, sample, 1.0 / sample_count);
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
    solid_angle_tables: &mut [Option<Vec<Real>>],
    face_size: u32,
    mip_count: u32,
    mip: u32,
    face_executor: &impl SourceMipmapFaceExecutor,
) {
    let mip_size = source_cubemap_mip_size(face_size, mip);
    let cone_angle = source_cubemap_angular_cone_angle(mip_size);
    let input_mip = source_cubemap_angular_input_mip(mip_count, cone_angle);
    let input_size = source_cubemap_mip_size(face_size, input_mip);
    let texel_solid_angles = solid_angle_tables[input_mip as usize].get_or_insert_with(|| {
        (0..input_size * input_size)
            .map(|index| {
                cubemap_texel_solid_angle(index % input_size, index / input_size, input_size)
            })
            .collect::<Vec<_>>()
    });
    let filter_face = |face, face_texels: &mut [[Real; 4]]| {
        for y in 0..mip_size {
            for x in 0..mip_size {
                let direction = cubemap_texel_direction(face, x, y, mip_size);
                face_texels[y as usize * mip_size as usize + x as usize] = integrate_angular_area(
                    average_mips,
                    face_size,
                    mip_count,
                    input_mip,
                    input_size,
                    texel_solid_angles,
                    direction,
                    cone_angle,
                );
            }
        }
    };
    let mut outputs = source_cubemap_face_mip_outputs(output_mips, face_size, mip_count, mip);
    // Match Unreal's >=128 source threshold and one-worker-per-face scheduling policy.
    if input_size >= 128 {
        face_executor.filter_faces(&mut outputs, &filter_face);
    } else {
        SerialSourceMipmapFaceExecutor.filter_faces(&mut outputs, &filter_face);
    }
}

pub(super) fn source_cubemap_angular_cone_angle(mip_size: u32) -> Real {
    (SOURCE_MIPMAP_MAX_CONE_ANGLE / mip_size.max(1) as Real)
        .clamp(SOURCE_MIPMAP_MIN_CONE_ANGLE, SOURCE_MIPMAP_MAX_CONE_ANGLE)
}

pub(super) fn source_cubemap_angular_input_mip(mip_count: u32, cone_angle: Real) -> u32 {
    let segment_height = SOURCE_MIPMAP_NORMALIZED_SPHERE_RADIUS * (1.0 - cone_angle.cos());
    let covered_area =
        2.0 * std::f32::consts::PI * SOURCE_MIPMAP_NORMALIZED_SPHERE_RADIUS * segment_height;
    let input_mip = 0.5 * covered_area.max(Real::EPSILON).log2() + mip_count as Real
        - SOURCE_MIPMAP_INPUT_QUALITY_BIAS;
    (input_mip.trunc() as i32).clamp(0, mip_count.max(1).saturating_sub(1) as i32) as u32
}

fn integrate_angular_area(
    average_mips: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    input_mip: u32,
    input_size: u32,
    texel_solid_angles: &[Real],
    filter_direction: [Real; 3],
    cone_angle: Real,
) -> [Real; 4] {
    let direction_threshold = cone_angle.cos().min(0.9999);
    let mut accumulator = AngularFilterAccumulator {
        average_mips,
        face_size,
        mip_count,
        input_mip,
        input_size,
        texel_solid_angles,
        filter_direction,
        cone_angle_sin: cone_angle.sin(),
        cone_angle_cos: cone_angle.cos(),
        direction_threshold,
        inverse_kernel_width: 1.0 / (1.0 - direction_threshold),
        color: [0.0; 4],
        weight_sum: 0.0,
    };

    for face in CubemapFace::ALL {
        let cone_axis = cubemap_side_space_direction(face, filter_direction);
        accumulator.traverse_face_region(face, cone_axis, 0, 0, input_size, input_size);
    }

    if accumulator.weight_sum <= Real::EPSILON {
        return sample_cubemap_linear_at_mip(
            average_mips,
            face_size,
            mip_count,
            filter_direction,
            input_mip,
        );
    }
    let inv_weight = 1.0 / accumulator.weight_sum;
    [
        accumulator.color[0] * inv_weight,
        accumulator.color[1] * inv_weight,
        accumulator.color[2] * inv_weight,
        accumulator.color[3] * inv_weight,
    ]
}

struct AngularFilterAccumulator<'a> {
    average_mips: &'a [[Real; 4]],
    face_size: u32,
    mip_count: u32,
    input_mip: u32,
    input_size: u32,
    texel_solid_angles: &'a [Real],
    filter_direction: [Real; 3],
    cone_angle_sin: Real,
    cone_angle_cos: Real,
    direction_threshold: Real,
    inverse_kernel_width: Real,
    color: [Real; 4],
    weight_sum: Real,
}

impl AngularFilterAccumulator<'_> {
    fn traverse_face_region(
        &mut self,
        face: CubemapFace,
        cone_axis: [Real; 3],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        if width == 0
            || height == 0
            || !self.face_region_intersects_cone(cone_axis, x, y, width, height)
        {
            return;
        }
        if width == 1 && height == 1 {
            self.accumulate_texel(face, x, y);
            return;
        }

        if width >= height && width > 1 {
            let first_width = width / 2;
            self.traverse_face_region(face, cone_axis, x, y, first_width, height);
            self.traverse_face_region(
                face,
                cone_axis,
                x + first_width,
                y,
                width - first_width,
                height,
            );
        } else {
            let first_height = height / 2;
            self.traverse_face_region(face, cone_axis, x, y, width, first_height);
            self.traverse_face_region(
                face,
                cone_axis,
                x,
                y + first_height,
                width,
                height - first_height,
            );
        }
    }

    fn face_region_intersects_cone(
        &self,
        cone_axis: [Real; 3],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> bool {
        let inverse_extent = 1.0 / self.input_size.max(1) as Real;
        let center = [
            (x as Real + width as Real * 0.5) * 2.0 * inverse_extent - 1.0,
            (y as Real + height as Real * 0.5) * 2.0 * inverse_extent - 1.0,
            1.0,
        ];
        let radius = SOURCE_MIPMAP_NODE_RADIUS_SCALE
            * ((width * width + height * height) as Real).sqrt()
            * inverse_extent;
        sphere_intersects_cone(
            center,
            radius,
            cone_axis,
            self.cone_angle_sin,
            self.cone_angle_cos,
        )
    }

    fn accumulate_texel(&mut self, face: CubemapFace, x: u32, y: u32) {
        let direction = cubemap_texel_direction(face, x, y, self.input_size);
        let direction_dot = dot3(self.filter_direction, direction);
        if direction_dot <= self.direction_threshold {
            return;
        }
        let kernel = (1.0 - (1.0 - direction_dot) * self.inverse_kernel_width).clamp(0.0, 1.0);
        let kernel = kernel * kernel * (3.0 - 2.0 * kernel);
        // Plan 06 intentionally keeps exact solid-angle weighting and filtered source alpha.
        let weight =
            kernel * self.texel_solid_angles[y as usize * self.input_size as usize + x as usize];
        let offset =
            source_cubemap_face_mip_offset(self.face_size, self.mip_count, face, self.input_mip);
        let texel = self.average_mips[offset + y as usize * self.input_size as usize + x as usize];
        add_weighted(&mut self.color, texel, weight);
        self.weight_sum += weight;
    }
}

fn sphere_intersects_cone(
    sphere_center: [Real; 3],
    sphere_radius: Real,
    cone_axis: [Real; 3],
    cone_angle_sin: Real,
    cone_angle_cos: Real,
) -> bool {
    let offset = scale3(
        cone_axis,
        -sphere_radius / cone_angle_sin.max(Real::EPSILON),
    );
    let difference = sub3(sphere_center, offset);
    let mut distance_squared = dot3(difference, difference);
    let mut projected_distance = dot3(cone_axis, difference);
    if projected_distance <= 0.0
        || projected_distance * projected_distance
            < distance_squared * cone_angle_cos * cone_angle_cos
    {
        return false;
    }

    distance_squared = dot3(sphere_center, sphere_center);
    projected_distance = -dot3(cone_axis, sphere_center);
    if projected_distance > 0.0
        && projected_distance * projected_distance
            >= distance_squared * cone_angle_sin * cone_angle_sin
    {
        distance_squared <= sphere_radius * sphere_radius
    } else {
        true
    }
}

fn dot3(first: [Real; 3], second: [Real; 3]) -> Real {
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}

fn scale3(value: [Real; 3], scale: Real) -> [Real; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn sub3(first: [Real; 3], second: [Real; 3]) -> [Real; 3] {
    [
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    ]
}

fn add_weighted(accumulator: &mut [Real; 4], texel: [Real; 4], weight: Real) {
    accumulator[0] += texel[0] * weight;
    accumulator[1] += texel[1] * weight;
    accumulator[2] += texel[2] * weight;
    accumulator[3] += texel[3] * weight;
}

#[cfg(test)]
mod tests {
    use super::{
        source_cubemap_mips_from_base, source_cubemap_mips_from_base_with_parallel_executor,
    };
    use crate::core::framework::tasks::ParallelSliceExecutor;
    use crate::core::math::Real;

    #[derive(Default)]
    struct CountingParallelSliceExecutor(std::sync::atomic::AtomicUsize);

    impl ParallelSliceExecutor for CountingParallelSliceExecutor {
        fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
        where
            T: Send,
            F: Fn(&mut [T]) + Send + Sync,
        {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            for chunk in items.chunks_mut(chunk_size.max(1)) {
                task(chunk);
            }
        }
    }

    #[test]
    fn parallel_angular_mipmap_writes_final_storage_and_matches_serial_output() {
        let face_size = 128;
        let mip_count = super::super::source_cubemap_mip_count(face_size);
        let base_texels = (0..super::super::source_cubemap_sample_count(face_size, mip_count))
            .map(|index| {
                let value = (index % 17) as Real / 16.0;
                [value, value * 0.5, 1.0 - value, 1.0]
            })
            .collect::<Vec<_>>();
        let serial = source_cubemap_mips_from_base(&base_texels, face_size, mip_count);

        let executor = CountingParallelSliceExecutor::default();
        let parallel = source_cubemap_mips_from_base_with_parallel_executor(
            &base_texels,
            face_size,
            mip_count,
            &executor,
        );

        assert_eq!(parallel, serial);
        assert!(
            executor.0.load(std::sync::atomic::Ordering::Relaxed) > 0,
            "a 128-face input must retain the angular face-parallel schedule"
        );
    }
}
